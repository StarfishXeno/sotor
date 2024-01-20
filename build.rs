use ahash::HashMap;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use sotor_internal::{util::Game, Appearance, Class, Feat, GameData, Quest};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

fn make_game_dir(common: &Path, name: &str) -> PathBuf {
    let mut dir = common.to_owned();
    dir.push(name);
    assert!(dir.exists(), "game directory missing at {dir:#?}");
    dir
}

fn write(out: &mut File, bytes: &[u8]) {
    out.write_all(bytes).unwrap();
}
fn writeln(out: &mut File, bytes: &[u8]) {
    write(out, bytes);
    out.write_all(b"\n").unwrap();
}
fn write_tokens(out: &mut File, stream: &TokenStream) {
    writeln(out, stream.to_string().as_bytes());
}

fn main() {
    dotenv::dotenv().unwrap();
    let mut steam_dir: PathBuf = dotenv::var("STEAM_LIBRARY").unwrap().into();
    steam_dir.push("steamapps");
    let mut common = steam_dir.clone();
    common.push("common");
    let game_dirs = [
        make_game_dir(&common, "swkotor"),
        make_game_dir(&common, "Knights of the Old Republic II"),
    ];

    let mut game_data = vec![];
    for game in Game::LIST {
        game_data.push(GameData::read(game, &game_dirs[game.idx()], Some(&steam_dir)).unwrap());
    }
    let game_data: [GameData; Game::COUNT] = game_data.try_into().unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let output_path = PathBuf::from_iter([&out_dir, "codegen.rs"]);
    let out = &mut std::fs::File::options()
        .write(true)
        .create(true)
        .open(output_path)
        .unwrap();

    write_tokens(
        out,
        &quote!(
            use ahash::{HashMap, HashMapExt as _};
            use sotor_internal::{util::Game, Appearance, Class, Feat, GameData, Item, Quest, QuestStage};
            use std::collections::BTreeMap;

            fn default_game_data() -> [GameData; Game::COUNT]
        ),
    );
    write(out, b"{[");
    for data in game_data {
        writeln(out, b"GameData {");
        write_feats("feats", out, data.feats);
        write_feats("powers", out, data.powers);
        write_classes(out, data.classes);
        write_appearances("portraits", out, data.portraits);
        write_appearances("appearances", out, data.appearances);
        write_appearances("soundsets", out, data.soundsets);
        write_quests(out, data.quests);
        writeln(out, b"items: HashMap::new()");

        writeln(out, b"},");
    }

    write(out, b"]}");
}

fn make_field(name: &str, content: &TokenStream) -> TokenStream {
    let ident = Ident::new(name, Span::call_site());
    quote!(#ident: #content,)
}

fn make_struct(name: &str, fields: &TokenStream) -> TokenStream {
    let ident = Ident::new(name, Span::call_site());
    quote!(
        #ident {
            #fields
        }
    )
}

fn write_map(out: &mut File, map: &str, content: Vec<(TokenStream, TokenStream)>) {
    let map_ident = Ident::new(map, Span::call_site());
    write_tokens(out, &quote!(#map_ident::from_iter));
    write(out, b"([");
    for (k, v) in content {
        write_tokens(out, &quote!((#k, #v),));
    }
    write(out, b"])");
}

fn make_option<T: ToTokens>(o: Option<T>, extra: &TokenStream) -> TokenStream {
    if o.is_some() {
        let v = o.unwrap();
        quote!(Some(#v #extra))
    } else {
        quote!(None)
    }
}

fn write_feats(field: &str, out: &mut File, feats: HashMap<u16, Feat>) {
    let feats = feats
        .into_iter()
        .map(|(id, feat)| {
            let mut fields = TokenStream::new();
            let name = feat.name;
            fields.extend(make_field("name", &quote!(#name.to_string())));
            fields.extend(make_field(
                "description",
                &make_option(feat.description, &quote!(.to_string())),
            ));
            (quote!(#id), make_struct("Feat", &fields))
        })
        .collect();

    write(out, format!("{field}: ").as_bytes());
    write_map(out, "HashMap", feats);
    writeln(out, b",");
}

fn write_classes(out: &mut File, classes: HashMap<i32, Class>) {
    let classes = classes
        .into_iter()
        .map(|(id, class)| {
            let mut fields = TokenStream::new();
            let name = class.name;
            let hit_die = class.hit_die;
            let force_die = class.force_die;
            fields.extend(make_field("name", &quote!(#name.to_string())));
            fields.extend(make_field("hit_die", &quote!(#hit_die)));
            fields.extend(make_field("force_die", &quote!(#force_die)));
            (quote!(#id), make_struct("Class", &fields))
        })
        .collect();

    write(out, b"classes:");
    write_map(out, "HashMap", classes);
    write(out, b",");
}

fn write_appearances(field: &str, out: &mut File, appearances: HashMap<u16, Appearance>) {
    let appearances = appearances
        .into_iter()
        .map(|(id, appearance)| {
            let mut fields = TokenStream::new();
            let name = appearance.name;
            fields.extend(make_field("name", &quote!(#name.to_string())));
            (quote!(#id), make_struct("Appearance", &fields))
        })
        .collect();

    write(out, format!("{field}: ").as_bytes());
    write_map(out, "HashMap", appearances);
    write(out, b",");
}

fn write_quests(out: &mut File, quests: HashMap<String, Quest>) {
    let quests = quests
        .into_iter()
        .map(|(id, quest)| {
            let mut fields = TokenStream::new();
            let name = quest.name;
            fields.extend(make_field("name", &quote!(#name.to_string())));
            let mut stage_tokens = TokenStream::new();
            for (id, stage) in quest.stages {
                let mut stage_fields = TokenStream::new();
                let end = stage.end;
                let descr = stage.description;
                stage_fields.extend(make_field("end", &quote!(#end)));
                stage_fields.extend(make_field("description", &quote!(#descr.to_string())));
                let s = make_struct("QuestStage", &stage_fields);

                stage_tokens.extend(quote!((#id, #s),));
            }

            fields.extend(make_field(
                "stages",
                &quote!(BTreeMap::from_iter([
                    #stage_tokens
                ])),
            ));
            (quote!(#id.to_string()), make_struct("Quest", &fields))
        })
        .collect();

    write(out, b"quests:");
    write_map(out, "HashMap", quests);
    write(out, b",");
}

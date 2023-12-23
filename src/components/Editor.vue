<script setup lang="ts">
import EditorGeneral from './EditorGeneral.vue';
import Button from './elements/Button.vue';
import { ref } from 'vue';

enum Tab {
    General = 0,
    Globals,
    Characters,
    Inventory,
}

const map = {
    [Tab.General]: { component: EditorGeneral, name: Tab[Tab.General] },
    [Tab.Globals]: { component: EditorGeneral, name: Tab[Tab.Globals] },
    [Tab.Characters]: { component: EditorGeneral, name: Tab[Tab.Characters] },
    [Tab.Inventory]: { component: EditorGeneral, name: Tab[Tab.Inventory] },
};
const tab = ref(Tab.General);
const setTab = (t: Tab) => (tab.value = t);
</script>

<template>
    <div class="tabs">
        <Button
            v-for="({ name }, value) in map"
            :key="value"
            :active="value == tab"
            class="tab"
            @click="setTab(value)"
        >
            {{ name }}
        </Button>

        <Button class="btn-save">Save</Button>
    </div>

    <div class="tab-container">
        <component :is="map[tab].component" />
    </div>
</template>
<style scoped lang="scss">
.tabs {
    display: flex;
}
.tab {
    margin-right: 10px;
}
.btn-save {
    margin-left: auto;
}
.tab-container {
    margin-top: 10px;
    border-radius: 10px;
    border: 2px solid globals.$green-dark;
    padding: 5px;
}
</style>

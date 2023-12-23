import { defineStore } from 'pinia';
import { readFromDirectory } from './util';
import { Save } from '../bindings/types';

export const useSaveStore = defineStore('save', {
    state: () => ({
        path: '',
        save: null as null | Save,
    }),
    actions: {
        async loadFromDirectory(path: string) {
            const save = await readFromDirectory(path);
            this.save = save;
            this.path = path;
        },
    },
});

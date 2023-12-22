import { defineStore } from 'pinia';

export const useSaveStore = defineStore('save', {
    state: () => ({
        save: null,
    }),
});

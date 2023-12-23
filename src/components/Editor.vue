<script setup lang="ts">
import { ref } from 'vue';
import EditorGeneral from './EditorGeneral.vue';
import Button from './elements/Button.vue';

enum Tab {
    General = 0,
    Globals,
    Characters,
    Inventory,
}

const components = [EditorGeneral, EditorGeneral, EditorGeneral, EditorGeneral];
const tabs = [Tab.General, Tab.Globals, Tab.Characters, Tab.Inventory].map((value, idx) => ({
    value,
    component: components[idx],
    name: Tab[value],
}));
const tab = ref(Tab.General);
const setTab = (t: Tab) => {
    tab.value = t;
};
</script>

<template>
    <div class="tabs">
        <Button
            v-for="{ name, value } in tabs"
            :key="value"
            :active="value === tab"
            class="tab"
            @click="setTab(value)"
        >
            {{ name }}
        </Button>

        <Button class="btn-save">Save</Button>
    </div>

    <div class="tab-container">
        <component :is="tabs[tab].component" />
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

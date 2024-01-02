<script setup lang="ts">
import Editor from './components/Editor.vue';
import useSaveStore from './store';

const store = useSaveStore();
store.loadFromDirectory('./save');
</script>

<template>
    <Editor v-if="store.save" />
</template>
<style></style>

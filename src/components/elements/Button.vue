<script lang="ts" setup>
interface TslButtonProps {
    active?: boolean;
}
const props = withDefaults(defineProps<TslButtonProps>(), { active: false });
</script>

<template>
    <button class="btn" :class="{ active: props.active }"><slot /></button>
</template>
<style scoped lang="scss">
.btn {
    background: rgba(globals.$black, 0.8);
    border: 2px solid globals.$green-dark;
    border-radius: 10px;
    padding: 8px;
    color: globals.$green;
}
.btn:hover {
    cursor: pointer;
    color: globals.$white;
    border-color: globals.$white;
}
.active {
    color: globals.$white;
    border-color: globals.$green;
}
</style>

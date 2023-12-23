<script lang="ts" setup>
interface InputProps {
    modelValue: string;
}
defineProps<InputProps>();
defineEmits(['update:modelValue']);
</script>

<template>
    <input
        class="input"
        :value="modelValue"
        @input="$emit('update:modelValue', (<HTMLInputElement>$event.target).value)"
    />
</template>
<style scoped lang="scss">
.input {
    background: globals.$white;
    border: 1px solid globals.$green-dark;
    border-radius: 5px;
    padding: 2px 5px;
    color: globals.$black;
}
.input:focus {
    outline: 2px solid globals.$green;
}
</style>

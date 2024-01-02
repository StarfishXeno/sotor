<script setup lang="ts">
import Input from './elements/Input.vue';
import useSaveStore from '../store';
import { formatTime } from '../util';

const { save } = useSaveStore();
const time = formatTime(save!.nfo.timePlayed);
</script>

<template>
    <table>
        <tr>
            <td class="label">Save name:</td>
            <td><Input v-model="save!.nfo.saveName" width="200px" /></td>
            <td class="label">Area name:</td>
            <td>{{ save!.nfo.areaName }}</td>
        </tr>

        <tr>
            <td class="label">Time played:</td>
            <td>{{ time }}</td>
            <td class="label">Cheats used:</td>
            <td><input type="checkbox" :checked="save!.nfo.cheatsUsed" /></td>
        </tr>
    </table>
</template>

<style scoped lang="scss">
td {
    color: globals.$white;
}
tr:not(:first-child) td {
    padding-top: 5px;
}
.label {
    color: globals.$green;
    padding-right: 10px;
    font-weight: bold;
}
.label:not(:first-child) {
    padding-left: 10px;
}
</style>

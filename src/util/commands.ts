import { invoke } from '@tauri-apps/api/tauri';
import { Save } from '../../bindings/types';

export const readFromDirectory = (path: string): Promise<Save> => {
    return invoke('read_from_directory', { path });
};

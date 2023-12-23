import { invoke } from '@tauri-apps/api/tauri';
import { Save } from '../../bindings/types';

export const readFromDirectory = (path: string): Promise<Save> => invoke('read_from_directory', { path });
export const saveToDirectory = (path: string, save: Save) => invoke('save_to_directory', { path, save });

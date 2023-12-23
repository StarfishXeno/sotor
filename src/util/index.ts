export * from './commands';

export const formatTime = (seconds: number) => {
    const secs = Math.floor(seconds % 60);
    const minutes = Math.floor((seconds / 60) % 60);
    const hours = Math.floor((seconds / 60 / 60) % 24);
    const days = Math.floor(seconds / 60 / 60 / 24);

    if (days > 0) {
        return `${days}d ${hours}h ${minutes}m ${secs}s`;
    }
    if (hours > 0) {
        return `${hours}h ${minutes}m ${secs}s`;
    }
    return `${minutes}m ${secs}s`;
};

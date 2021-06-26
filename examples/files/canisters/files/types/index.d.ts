export type FileMeta = Readonly<{
    id: string;
    name: string;
    numChunks: number;
}>;

export type ChunkInfo = Readonly<{
    startByte: number;
    endByte: number;
}>;

export type UploadState = Readonly<{
    uploading: boolean;
    percentage: number;
    totalChunks: number;
    totalChunksUploaded: number;
}>;

export type DownloadState = Readonly<{
    downloading: boolean;
    percentage: number;
    totalChunks: number;
    totalChunksDownloaded: number;
}>;
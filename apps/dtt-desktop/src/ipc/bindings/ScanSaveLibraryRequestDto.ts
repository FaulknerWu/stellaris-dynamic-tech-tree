import type { SaveLibrarySourceDto } from "./SaveLibrarySourceDto";

export type ScanSaveLibraryRequestDto = {
  source: SaveLibrarySourceDto;
  documentsDir?: string | undefined;
  steamLibraries: Array<string>;
};

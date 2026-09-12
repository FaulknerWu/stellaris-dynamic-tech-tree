import type { SaveIndexStateDto } from "./SaveIndexStateDto";
import type { SaveMetadataDto } from "./SaveMetadataDto";

export type SaveIndexDto = {
  path: string;
  fileName: string;
  metadata?: SaveMetadataDto | undefined;
  modifiedAtMillis: number;
  fileSize: number;
  state: SaveIndexStateDto;
  unavailableReason?: string | undefined;
};

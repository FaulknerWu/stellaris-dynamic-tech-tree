import type { SaveIndexDto } from "./SaveIndexDto";

export type SaveCampaignDto = {
  key: string;
  saves: Array<SaveIndexDto>;
};

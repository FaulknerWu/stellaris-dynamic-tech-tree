import type { PlayerCountryCandidateDto } from "./PlayerCountryCandidateDto";
import type { SnapshotDto } from "./SnapshotDto";

export type InspectedSaveDto = {
  snapshot?: SnapshotDto | undefined;
  playerCountries: Array<PlayerCountryCandidateDto>;
};

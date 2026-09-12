import type { GovernmentDto } from "./GovernmentDto";
import type { SpeciesDto } from "./SpeciesDto";

export type SnapshotDto = {
  ethics: Array<string>;
  government: GovernmentDto;
  ascensionPerks: Array<string>;
  traditions: Array<string>;
  founderSpecies: SpeciesDto;
  countryType: string;
};

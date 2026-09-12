export type GenerationStageDto =
  | "SAVE_PARSE"
  | "LOAD_ORDER"
  | "INGEST_TECH"
  | "RELATIONS"
  | "INGEST_L10N"
  | "RENDER"
  | "CYCLES"
  | "WRITE_OUTPUT"
  | "DONE";

import "i18next";
import type { EnglishResources } from "./resource-types.generated";
declare module "i18next" {
  interface CustomTypeOptions {
    defaultNS: "common";
    resources: EnglishResources;
    strictKeyChecks: true;
    enableSelector: true;
    returnNull: false;
  }
}

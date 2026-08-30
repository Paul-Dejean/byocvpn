import type { Region } from "../bindings";

export type { Region };

export interface RegionGroup {
  continent: string;
  regions: Region[];
}

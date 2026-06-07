export enum EnableRegionEvent {
  Progress = "enable-region-progress",
  Complete = "enable-region-complete",
  Failed = "enable-region-failed",
}

export interface Region {
  name: string;
  country: string;
}

export interface RegionGroup {
  continent: string;
  regions: Region[];
}

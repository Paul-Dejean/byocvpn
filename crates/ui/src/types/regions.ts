export interface Region {
  name: string;
  country: string;
}

export interface RegionGroup {
  continent: string;
  regions: Region[];
}

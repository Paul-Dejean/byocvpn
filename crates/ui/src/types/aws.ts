export interface AwsRegion {
  name: string;
  country: string;
}

export interface RegionGroup {
  continent: string;
  regions: AwsRegion[];
}

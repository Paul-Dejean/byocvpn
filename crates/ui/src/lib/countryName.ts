const REGION_DISPLAY_NAMES = new Intl.DisplayNames(["en"], { type: "region" });

export function getCountryName(countryCode: string): string {
  if (!countryCode) {
    return "";
  }
  try {
    return REGION_DISPLAY_NAMES.of(countryCode.toUpperCase()) ?? "";
  } catch (error) {
    return "";
  }
}

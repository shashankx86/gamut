export interface Version {
  label: string;
  url: string;
}

// Add new versions at the TOP of this array. First entry = current release.
export const versions: Version[] = [
  { label: 'v0.1.2', url: '/docs' },
];

export const latestVersion = versions[0];

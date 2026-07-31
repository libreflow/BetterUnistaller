export type Scope = "machine" | "user";

export interface Program {
  id: string;
  name: string;
  publisher: string | null;
  version: string | null;
  installDate: string | null; // "YYYY-MM-DD"
  estimatedSizeBytes: number | null;
  installLocation: string | null;
  uninstallString: string | null;
  quietUninstallString: string | null;
  displayIcon: string | null;
  iconDataUri?: string;
  scope: Scope;
  isSystemEntry: boolean;
}

import { create } from "zustand";

type DirectoryStoreProps = {
  directory: string;
  setDirectory: (newDirectory: string) => void;
};

export const directoryStore = create<DirectoryStoreProps>((set) => ({
  directory: "",
  setDirectory: (newDirectory: string) =>
    set(() => ({ directory: newDirectory })),
}));

import { create } from "zustand";

type DirectoryStoreProps = {
  selectedDirectory: string;
  setSelectedDirectory: (newDirectory: string) => void;
};

export const directoryStore = create<DirectoryStoreProps>((set) => ({
  selectedDirectory: "",
  setSelectedDirectory: (newDirectory: string) =>
    set(() => ({ selectedDirectory: newDirectory })),
}));

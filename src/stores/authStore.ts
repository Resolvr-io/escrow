import { create } from "zustand";
import { devtools } from "zustand/middleware";

export interface AuthState {
  pubkey: string;
  setPubkey: (pubkey: string) => void;
}

const useAuthStore = create<AuthState>()(
  devtools((set) => ({
    pubkey: "",
    setPubkey: (pubkey: string) => set({ pubkey }),
  })),
);

export default useAuthStore;

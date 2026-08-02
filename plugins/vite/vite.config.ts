import type { UserConfig, Plugin } from "vite";
import react from "@vitejs/plugin-react";
import vue from "@vitejs/plugin-vue";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import meld from "./@meld";

const config: UserConfig = {
  plugins: [
    meld({ include: /\.meld$/, minifyHtml: false }),

    react({ include: ["**/*.jsx", "**/*.tsx"] }),
    vue({ include: "**/*.vue" }),
    svelte({ include: "**/*.svelte" }),
  ],
  build: {
    manifest: true,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules/react")) {
            return "react-vendor";
          }
          if (id.includes("node_modules/vue")) {
            return "vue-vendor";
          }
          if (id.includes("node_modules/svelte")) {
            return "svelte-vendor";
          }
        },
      },
    },
  },
};

export default config;

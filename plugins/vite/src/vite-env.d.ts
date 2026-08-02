/// <reference types="vite/client" />

declare module "*.svelte" {
  export { SvelteComponent as default };
  import type { SvelteComponent } from "svelte";
}

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module "*.react" {
  import type { FC } from "react";
  const component: FC<any>;
  export default component;
}

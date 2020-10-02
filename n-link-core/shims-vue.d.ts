declare module '*.vue' {
  import Vue from 'vue'
  export default Vue
}

import Vue, { PluginFunction } from 'vue';

interface IAsyncComputedOptions {
  errorHandler?: (error: string[]) => void;
  useRawError?: boolean;
  default?: any;
}

export default class AsyncComputed {
  constructor(options?: IAsyncComputedOptions);
  static install: PluginFunction<never>;
  static version: string;
}

type AsyncComputedGetter<T> = () => Promise<T> | T;
export interface IAsyncComputedProperty<T> {
  default?: T | (() => T);
  get: AsyncComputedGetter<T>;
  watch?: () => void;
  shouldUpdate?: () => boolean;
  lazy?: boolean;
}

interface IAsyncComputedProperties {
  [K: string]: AsyncComputedGetter<any> | IAsyncComputedProperty<any>;
}

declare module 'vue/types/options' {
  interface ComponentOptions<V extends Vue> {
    // @ts-ignore
    asyncComputed?: IAsyncComputedProperties;
  }
}

interface IASyncComputedState {
  state: 'updating' | 'success' | 'error';
  updating: boolean;
  success: boolean;
  error: boolean;
  exception: Error | null;
  update: () => void;
}

declare module 'vue/types/vue' {
  // tslint:disable-next-line:interface-name
  interface Vue {
    $asyncComputed: { [K: string]: IASyncComputedState };
  }
}

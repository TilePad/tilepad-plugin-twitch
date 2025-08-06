/**
 * Event emitting and subscribing
 */
export declare class EventEmitter {
  events: Record<string, any>;
  constructor();
  on(event: any, callback: any): void;
  off(event: any, callback: any): void;
  emit(event: any, ...args: any[]): void;
}

declare class Inspector extends EventEmitter {
  constructor();
  /**
   * Sends a message to the inspector
   *
   * @param msg The message to send
   */
  send(msg: unknown): void;
}

export declare const inspector: Inspector;

declare const plugin: {
  /**
   * Send a message to the plugin
   *
   * @param message The message to send
   */
  send(message: unknown): void;
  /**
   * Subscribes to messages sent to the inspector via the
   * associated plugin for the action
   *
   * The returned function can be used to remove the subscription
   *
   * @param callback The callback to invoke when a message is received
   * @returns Function that will remove the listener when called
   */
  onMessage(callback: (message: unknown) => void): () => void;
  requestProperties(): void;
  onProperties(callback: (properties: unknown) => void): () => void;
  getProperties(): Promise<unknown>;
  setProperty: (...args: any[]) => void;
  setProperties(properties: unknown, partial?: boolean): void;
};

interface Tile {
  profileId: string;
  folderId: string;
  pluginId: string;
  tileId: string;
  actionId: string;
  properties: unknown;
}

export type TilepadLabel = Partial<{
  enabled: boolean;
  label: string;
  align: "Bottom" | "Middle" | "Top";
  font_size: number;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  outline: boolean;
  color: string;
  outline_color: string;
}>;

export type TilepadIcon =
  | {
      type: "None";
    }
  | {
      type: "PluginIcon";
      plugin_id: string;
      icon: string;
    }
  | {
      type: "IconPack";
      pack_id: string;
      path: string;
    }
  | {
      type: "Url";
      src: string;
    };

declare const tile: {
  /**
   * Request the current tile details
   */
  requestTile(): void;
  /**
   * Get the current tile details
   */
  getTile(): Promise<Tile>;
  /**
   * Subscribes to tile, will receive the outcome
   * of {@link Tilepad.requestTile}
   *
   * The returned function can be used to remove the subscription
   *
   * @param callback The callback to invoke when a message is received
   * @returns Function that will remove the listener when called
   */
  onTile: (callback: (tile: Tile) => void) => () => void;
  /**
   * Requests the current properties for the tile.
   * When the properties are received {@link Tilepad.onProperties}
   * will be run
   */
  requestProperties(): void;
  /**
   * Subscribes to properties for the tile, will receive the outcome
   * of {@link Tilepad.requestProperties}
   *
   * The returned function can be used to remove the subscription
   *
   * @param callback The callback to invoke when a message is received
   * @returns Function that will remove the listener when called
   */
  onProperties(callback: (properties: unknown) => void): () => void;
  /**
   * Requests the current properties waiting until they're
   * obtained returning a promise that resolves with the
   * properties
   */
  getProperties(): Promise<unknown>;
  /**
   * Set a property within the tile properties
   *
   * @param name The name of the property to set
   * @param value The value of the property
   */
  setProperty: (...args: any[]) => void;
  /**
   * Sets the properties of the tile.
   *
   * This is a partial update, only the provided parts
   * of the object will be updated, anything not specified
   * already existing in the tile properties will continue
   * to exist
   *
   * @param properties The partial tile properties data
   */
  setProperties(properties: unknown): void;
  /**
   * Set the current label of the tile. Will not
   * work if the user has already specified a label
   * user must make their label blank for this to apply
   *
   * @param label
   */
  setLabel(label: TilepadLabel): void;
  /**
   * Set the current icon of the tile. Will not
   * work if the user has already specified a icon
   * user must set their icon to None for this to apply
   *
   * @param icon
   */
  setIcon(icon: TilepadIcon): void;
};

/**
 * Helper to debounce calls to a function to ensure that
 * a delay has elapsed between calls
 *
 * @param {*} fn The function to call
 * @param {*} delay The delay to wait before calling (Reset if called before the delay has elapsed)
 * @returns The debounced function
 */
export declare function debounce(fn: any, delay: any): (...args: any[]) => void;

declare global {
  declare const tilepad: {
    tile: typeof tile;
    plugin: typeof plugin;
  };
}

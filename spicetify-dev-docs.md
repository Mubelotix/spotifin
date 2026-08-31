# Spicetify Development Docs (merged)

Merged from https://github.com/spicetify/docs/tree/main/docs/development on 2026-08-26
(spicetify CLI v2.44 era). Each source file is delimited by a line '=== <path> ==='
so sections can be located with grep. Source of truth remains the upstream repo.

=== docs/development/api-wrapper/classes/context-menu.md ===
---
title: ContextMenu
description: Create custom menu item and prepend to right click context menu.
---

Create custom menu item and prepend to right click context menu.

Useful for adding custom actions to context menu (when a user right-clicks on a track, album, artist, etc.)

## Classes

### Item

Single context menu item.


```ts
new Spicetify.ContextMenu.Item(
  name: string,
  onClick: OnClickCallback,
  shouldAdd?: ShouldAddCallback,
  icon?: SVGIcon | string,
  disabled?: boolean,
)
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |
| onClick | `OnClickCallback` | Callback function when the menu item is clicked. |
| shouldAdd | `ShouldAddCallback` | Callback function to determine if the menu item should be added. |
| icon | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) &#124; `string` | Icon of the menu item. |
| disabled | `boolean` | Whether the menu item is disabled. |

#### Properties

:::tip

All of the listed properties are dynamic and can be changed at any time. Look into the example below for more information.

:::

| Name | Type | Description |
| :--- | :--- | :--- |
| iconList | [`readonly SVGIcon[]`](/docs/development/api-wrapper/types/svgicon) | List of icons. |
| name | `string` | Name of the menu item. |
| icon | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) &#124; `string` | Icon at the end of the menu item. |
| disabled | `boolean` | Whether the menu item is disabled. |
| shouldAdd | [`ShouldAddCallback`](/docs/development/api-wrapper/types/context-menu/should-add-callback) | Callback function to determine if the menu item should be added. |
| onClick | [`OnClickCallback`](/docs/development/api-wrapper/types/context-menu/onclick-callback) | Callback function when the menu item is clicked. |

#### Methods

##### `register`

Register the menu item to context menu.

```ts
register(): void
```

##### `deregister`

Remove the menu item from context menu.

```ts
deregister(): void
```
#### Example

```ts
// This function will determine if the selected item is a track
function ifItemIsTrack(uri) {
  let uriObj = Spicetify.URI.fromString(uri[0]);
  switch (uriObj.type) {
    case Type.TRACK:
      return true;
  }
  return false;
}

// Create a new menu item that only appears when a track is selected
const menuItem = new Spicetify.ContextMenu.Item(
  "My Menu Item",
  () => {
    Spicetify.showNotification("My Menu Item clicked!");
  },
  ifItemIsTrack,
  Spicetify.SVGIcons["play"],
  false,
);

// Register the menu item
menuItem.register();

// Deregister the menu item
menuItem.deregister();

// Change the menu item's name
menuItem.name = "My New Menu Item";

// Change the menu item's icon
menuItem.icon = "pause"
```

### SubMenu

Create a sub menu to contain `Item`s.

`Item`s in `subItems` array shouldn't be registered.


```ts
new Spicetify.ContextMenu.SubMenu(
  name: string,
  subItems: Iterable<Item>,
  shouldAdd?: ShouldAddCallback,
  disabled?: boolean,
)
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |
| subItems | [`Iterable<Item>`](#item) | Array of `Item`s to be added to the sub menu. |
| shouldAdd | [`ShouldAddCallback`](/docs/development/api-wrapper/types/context-menu/should-add-callback) | Callback function to determine if the menu item should be added. |
| disabled | `boolean` | Whether the menu item is disabled. |

#### Properties

:::tip

All of the listed properties are dynamic and can be changed at any time. Look into the example below for more information.

:::

| Name | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |
| disabled | `boolean` | Whether the menu item is disabled. |
| shouldAdd | [`ShouldAddCallback`](/docs/development/api-wrapper/types/context-menu/should-add-callback) | Callback function to determine if the menu item should be added. |

#### Methods

##### `addItem`

Add an `Item` to the sub menu.

```ts
addItem(item: Item): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| item | [`Item`](#item) | `Item` to be added to the sub menu. |

##### `removeItem`

Remove an `Item` from the sub menu.

```ts
removeItem(item: Item): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| item | [`Item`](#item) | `Item` to be removed from the sub menu. |

##### `register`

Register the sub menu to context menu.

```ts
register(): void
```

##### `deregister`

Remove the sub menu from context menu.

```ts
deregister(): void
```

#### Example

```ts
// Create a new menu item
const menuItem = new Spicetify.ContextMenu.Item(
  "My Menu Item",
  () => {
    Spicetify.showNotification("My Menu Item clicked!");
  },
  () => true,
  Spicetify.SVGIcons["play"],
  false,
);

// Create a new sub menu
const subMenu = new Spicetify.ContextMenu.SubMenu(
  "My Sub Menu",
  [menuItem],
  () => true,
  false,
);

// Register the sub menu
subMenu.register();

// Deregister the sub menu
subMenu.deregister();

// Change the sub menu's name
subMenu.name = "My New Sub Menu";

// Add a new menu item to the sub menu
subMenu.addItem(new Spicetify.ContextMenu.Item(
  "My New Menu Item",
  () => {
    Spicetify.showNotification("My New Menu Item clicked!");
  },
  () => true,
  Spicetify.SVGIcons["play"],
  false,
));
```


=== docs/development/api-wrapper/classes/menu.md ===
---
title: Menu
description: Create and prepend custom menu items in the profile menu.
---

Create and prepend custom menu items in the profile menu.

## Classes

### Item

Single menu item.

```ts
new Spicetify.Menu.Item(
  name: string,
  isEnabled: boolean,
  onClick: (self: Item) => void,
  icon?: SVGIcon | string,
)
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |
| isEnabled | `boolean` | Whether the menu item is enabled. |
| onClick | `(self: Item) => void` | Callback function when the menu item is clicked. |
| icon | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) &#124; `string` | Icon of the menu item. |

#### Properties

:::tip

All of the listed properties are dynamic and can be changed at any time. Look into the example below for more information.

:::

| Name | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |
| isEnabled | `boolean` | Whether the menu item is enabled. |
| icon | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) &#124; `string` | Icon of the menu item. |

#### Methods

##### setName

Set the label of the menu item.

```ts
setName(name: string): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |

##### setState

Set the state of the menu item. The item will have a tick icon next to it if its state is enabled.

```ts
setState(isEnabled: boolean): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| isEnabled | `boolean` | Whether the menu item is enabled. |

##### setIcon

Set the icon at the end of the menu item.

```ts
setIcon(icon: SVGIcon | string): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| icon | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) &#124; `string` | Icon of the menu item. |

##### register

Register the menu item to the profile menu.

```ts
register(): void
```

##### deregister

Remove the menu item from the profile menu.

```ts
deregister(): void
```

#### Example

```ts
const item = new Spicetify.Menu.Item("My Item", true, () => {
  console.log("My Item is clicked");
});

item.register();

// item.name = "My Item (Updated)";
item.setName("My Item (Updated)");

// item.isEnabled = false;
item.setState(false);

// item.icon = "heart";
item.setIcon("heart");
```

### SubMenu

Create a sub menu to contain `Item` toggles.

`Item`s in `subItems` array shouldn't be registered.

```ts
new Spicetify.Menu.SubMenu(
  name: string,
  subItems: Item[],
)
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |
| subItems | [`Item[]`](/docs/development/api-wrapper/classes/menu#item) | Array of sub menu items. |

#### Properties

:::tip

All of the listed properties are dynamic and can be changed at any time. Look into the example below for more information.

:::

| Name | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |

#### Methods

##### setName

Set the label of the menu item.

```ts
setName(name: string): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| name | `string` | Name of the menu item. |

##### addItem

Add a sub menu item.

```ts
addItem(item: Item): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| item | [`Item`](/docs/development/api-wrapper/classes/menu#item) | Sub menu item. |

##### removeItem

Remove a sub menu item.

```ts
removeItem(item: Item): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| item | [`Item`](/docs/development/api-wrapper/classes/menu#item) | Sub menu item. |

##### register

Register the menu item to profile menu.

```ts
register(): void
```

##### deregister

Remove the menu item from profile menu.

```ts
deregister(): void
```

#### Example

```ts
const item1 = new Spicetify.Menu.Item("My Item 1", true, () => {
  console.log("My Item 1 is clicked");
});

const item2 = new Spicetify.Menu.Item("My Item 2", true, () => {
  console.log("My Item 2 is clicked");
});

const subMenu = new Spicetify.Menu.SubMenu("My Sub Menu", [item1, item2]);

subMenu.register();

// subMenu.name = "My Sub Menu (Updated)";
subMenu.setName("My Sub Menu (Updated)");

// subMenu.addItem(item3);
subMenu.addItem(
  new Spicetify.Menu.Item("My Item 3", true, () => {
    console.log("My Item 3 is clicked");
  })
);
```


=== docs/development/api-wrapper/classes/playbar.md ===
---
title: Playbar
description: Create buttons in the player.
---

Create buttons in the player.

```ts
namespace Playbar {
    class Button {
        constructor(label: string, icon: SVGIcon | string, onClick: (self: Button) => void, disabled?: boolean, active?: boolean, registerOnCreate?: boolean);
        label: string;
        icon: string;
        onClick: (self: Button) => void;
        disabled: boolean;
        active: boolean;
        element: HTMLButtonElement;
        tippy: any;
        register: () => void;
        deregister: () => void;
    }

    class Widget {
        constructor(label: string, icon: SVGIcon | string, onClick?: (self: Widget) => void, disabled?: boolean, active?: boolean, registerOnCreate?: boolean);
        label: string;
        icon: string;
        onClick: (self: Widget) => void;
        disabled: boolean;
        active: boolean;
        element: HTMLButtonElement;
        tippy: any;
        register: () => void;
        deregister: () => void;
    }
};
```

## `Button`

Create buttons next to the player extra control buttons (e.g. queue, lyrics, Now Playing View, etc.).

This is useful for creating buttons whose actions have an impact on or relate to the player, and are generally dynamic/stateful, such as a button that toggles the player's loop mode.

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| label | `string` | Label of the button. |
| icon | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) &#124; `string` | Icon of the button. |
| onClick | `(self: Button) => void` | Callback function when the button is clicked. |
| disabled | `boolean` &#124; `undefined` | Whether the button is disabled. |
| active | `boolean` &#124; `undefined` | Whether the button is active. |
| registerOnCreate | `boolean` &#124; `undefined` | Whether the button should be registered to the player on creation. |

#### Properties

:::tip

All of the listed properties are dynamic and can be changed at any time. Look into the example below for more information.

:::

| Name | Type | Description |
| :--- | :--- | :--- |
| label | `string` | Label of the button. |
| icon | `string` | Icon of the button. |
| disabled | `boolean` | Whether the button is disabled. |
| active | `boolean` | Whether the button is active. |
| onClick | `(self: Button) => void` | Callback function when the button is clicked. |
| element | `HTMLButtonElement` | HTML element of the button. |
| tippy | `any` | Tippy instance of the button. For more information, see [Tippy.js](https://atomiks.github.io/tippyjs/v6/tippy-instance/). |

#### Methods

##### `register`

Register the button to the player.

```ts
register(): void;
```

##### `deregister`

Deregister the button from the player.

```ts
deregister(): void;
```

#### Example

:::caution

Tippy, `onclick` or any other click events will **not** work if `disabled` is set to `true`. You will need to manually enable the button inside your extension.

This is due to the limitations of Tippy itself and how HTML elements work.

:::

```ts
// By default, the button will be registered to the player on creation.
// You can disable this by setting registerOnCreate to false.
// Each button comes with a preconfigured Tippy instance that aims to mimic the original Spotify tooltip.
const button = new Spicetify.Playbar.Button(
    "My Button",
    "play",
    (self) => {
        // Do something when the button is clicked.
    },
    false, // Whether the button is disabled.
    false, // Whether the button is active.
);

// You can also register the button to the player later.
button.register();

// Remove the button from the player when it is no longer needed.
button.deregister();
// If you don't want to remove the button entirely, you can also disable it.
button.disabled = true;

// Change button properties.
// Changing label will also change the tooltip content.
button.label = "Hello world!";
button.icon = "play";

// You can also set properties of the HTML element.
button.element.style.color = "red";
button.element.oncontextmenu = () => {
    Spicetify.showNotification("You right-clicked me!");
};
button.element.addEventListener("click", () => {
    // Do something else.
    Spicetify.showNotification("You clicked me!");
});

// You can also change properties of the Tippy instance. For more information, see https://atomiks.github.io/tippyjs/v6/tippy-instance/.
button.tippy.setContent("Hello world!");

// Or if you want to use HTML.
button.tippy.setProps({
    content: "<b>Hello world!</b>",
    allowHTML: true,
});
```

## `Widget`

Create widgets in the player, next to track information similar to the Heart button.

This is useful for creating buttons whose actions have an impact on the state of the player and the track being played, such as a button that adds the current track to a playlist.

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| label | `string` | Label of the widget. |
| icon | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) &#124; `string` | Icon of the widget. |
| onClick | `(self: Widget) => void` | Callback function when the widget is clicked. |
| disabled | `boolean` &#124; `undefined` | Whether the widget is disabled. |
| active | `boolean` &#124; `undefined` | Whether the widget is active. |
| registerOnCreate | `boolean` &#124; `undefined` | Whether the widget should be registered to the player on creation. |

#### Properties

:::tip

All of the listed properties are dynamic and can be changed at any time. Look into the example below for more information.

:::

| Name | Type | Description |
| :--- | :--- | :--- |
| label | `string` | Label of the widget. |
| icon | `string` | Icon of the widget. |
| disabled | `boolean` | Whether the widget is disabled. |
| active | `boolean` | Whether the widget is active. |
| onClick | `(self: Widget) => void` | Callback function when the widget is clicked. |
| element | `HTMLButtonElement` | HTML element of the widget. |
| tippy | `any` | Tippy instance of the widget. For more information, see [Tippy.js](https://atomiks.github.io/tippyjs/v6/tippy-instance/). |

#### Methods

##### `register`

Register the widget to the player.

```ts
register(): void;
```

##### `deregister`

Deregister the widget from the player.

```ts
deregister(): void;
```

#### Example

:::caution

Tippy, `onclick` or any other click events will **not** work if `disabled` is set to `true`. You will need to manually enable the widget inside your extension.

This is due to the limitations of Tippy itself and how HTML elements work.

:::

```ts
// By default, the widget will be registered to the player on creation.
// You can disable this by setting registerOnCreate to false.
// Each widget comes with a preconfigured Tippy instance that aims to mimic the original Spotify tooltip.
const widget = new Spicetify.Playbar.Widget(
    "My Widget",
    "play",
    (self) => {
        // Do something when the widget is clicked.
    },
    false, // Whether the widget is disabled.
    false, // Whether the widget is active.
);

// You can also register the widget to the player later.
widget.register();

// Remove the widget from the player when it is no longer needed.
widget.deregister();

// Change widget properties.
// Changing label will also change the tooltip content.
widget.label = "Hello world!";
widget.icon = "play";

// You can also set properties of the HTML element.
widget.element.style.color = "red";
widget.element.oncontextmenu = () => {
    Spicetify.showNotification("You right-clicked me!");
};
widget.element.addEventListener("click", () => {
    // Do something else.
    Spicetify.showNotification("You clicked me!");
});

// You can also change properties of the Tippy instance. For more information, see https://atomiks.github.io/tippyjs/v6/tippy-instance/.
widget.tippy.setContent("Hello world!");

// Or if you want to use HTML.
widget.tippy.setProps({
    content: "<b>Hello world!</b>",
    allowHTML: true,
});
```


=== docs/development/api-wrapper/classes/topbar.md ===
---
title: Topbar
description: Create buttons in the top bar.
---

Create buttons in the top bar, next to the navigation buttons.

This is useful for creating buttons that are generally static and whose actions have an impact on the whole app, such as a button that opens a settings menu.

```ts
namespace Topbar {
    class Button {
        constructor(label: string, icon: SVGIcon | string, onClick: (self: Button) => void, disabled?: boolean, isRight?: boolean);
        label: string;
        icon: string;
        onClick: (self: Button) => void;
        disabled: boolean;
        isRight: boolean;
        element: HTMLButtonElement;
        tippy: any;
    }
};
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| label | `string` | Label of the button. |
| icon | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) &#124; `string` | Icon of the button. |
| onClick | `(self: Button) => void` | Callback function when the button is clicked. |
| disabled | `boolean` | Whether the button is disabled. |
| isRight | `boolean` | Whether the button is button placed on the right side. |

#### Properties

:::tip

All of the listed properties are dynamic and can be changed at any time. Look into the example below for more information.

:::

| Name | Type | Description |
| :--- | :--- | :--- |
| label | `string` | Label of the button. |
| icon | `string` | Icon of the button. |
| disabled | `boolean` | Whether the button is disabled. |
| onClick | `(self: Button) => void` | Callback function when the button is clicked. |
| element | `HTMLButtonElement` | HTML element of the button. |
| tippy | `any` | Tippy instance of the button. For more information, see [Tippy.js](https://atomiks.github.io/tippyjs/v6/tippy-instance/). |

#### Example

:::caution

Tippy, `onclick` or any other click events will **not** work if `disabled` is set to `true`. You will need to manually enable the button inside your extension.

This is due to the limitations of Tippy itself and how HTML elements work.

:::

```ts
// Button is automatically added to the top bar when created.
// Each button comes with a preconfigured Tippy instance that aims to mimic the original Spotify tooltip.
const button = new Spicetify.Topbar.Button("Hello", "download", () => {
    Spicetify.showNotification("Hello world!");
});

// Change button properties.
// Changing label will also change the tooltip content.
button.label = "Hello world!";
button.icon = "play";
button.disabled = true;

// You can also set properties of the HTML element.
button.element.style.color = "red";
button.element.oncontextmenu = () => {
    Spicetify.showNotification("You right-clicked me!");
};
button.element.addEventListener("click", () => {
    // Do something else.
    Spicetify.showNotification("You clicked me!");
});

// You can also change properties of the Tippy instance. For more information, see https://atomiks.github.io/tippyjs/v6/tippy-instance/.
button.tippy.setContent("Hello world!");

// Or if you want to use HTML.
button.tippy.setProps({
    content: "<b>Hello world!</b>",
    allowHTML: true,
});
```


=== docs/development/api-wrapper/functions/add-to-queue.md ===
---
title: addToQueue
description: Adds a track/album or array of tracks/albums to prioritized queue.
---

Adds a track or array of tracks to the bottom of the prioritized queue.

:::tip

This works similarly to [`Spicetify.Platform.PlayerAPI.addToQueue`](/docs/development/api-wrapper/methods/platform#addtoqueue) but works silently, meaning no notification will be shown.

If you want default Spotify behavior, use [`Spicetify.Platform.PlayerAPI.addToQueue`](/docs/development/api-wrapper/methods/platform#addtoqueue) instead.

:::

```ts
function addToQueue(uri: ContextTrack[]): Promise<void>;
```

#### Parameters

| Name | Type | Description |
| :--- | :--- | :--- |
| `uri` | [`ContextTrack[]`](/docs/development/api-wrapper/types/context-track) | Array of tracks to add to queue. |

#### Example

```ts
// Add current track to queue
const currentTrack = Spicetify.Player.data.item;

await Spicetify.addToQueue([currentTrack]);

// Add a track to queue
const trackUri = "spotify:track:4iV5W9uYEdYUVa79Axb7Rh";

await Spicetify.addToQueue([ { uri: trackUri } ]);


=== docs/development/api-wrapper/functions/color-extractor.md ===
---
title: colorExtractor
description: Extracts colors from a playlist, track, album, artist, show, etc.
---

Extracts colors from a playlist, track, album, artist, show, etc.

```ts
function colorExtractor(uri: string): Promise<{
    DARK_VIBRANT: string;
    DESATURATED: string;
    LIGHT_VIBRANT: string;
    PROMINENT: string;
    VIBRANT: string;
    VIBRANT_NON_ALARMING: string;
}>;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| uri | `string` | URI of anything that has artwork (playlist, track, album, artist, show, etc.) |

#### Returns

| Name | Type | Description |
| :--- | :--- | :--- |
| DARK_VIBRANT | `string` | Dark vibrant color in hex format. |
| DESATURATED | `string` | Desaturated color in hex format. |
| LIGHT_VIBRANT | `string` | Light vibrant color in hex format. |
| PROMINENT | `string` | Prominent color in hex format. |
| VIBRANT | `string` | Vibrant color in hex format. |
| VIBRANT_NON_ALARMING | `string` | Vibrant non alarming color in hex format. |

#### Example

```ts
// Get color from current track
const currentTrack = Spicetify.Player.data.item;
const colors = await Spicetify.colorExtractor(currentTrack.uri);
```


=== docs/development/api-wrapper/functions/get-audio-data.md ===
---
title: getAudioData
description: Get the audio data from a track.
---

Fetch track analyzed audio data.

Under the hood, it uses the `wg://audio-attributes/v1/audio-analysis/` endpoint, which is identical to Spotify Web API's [Get Track's Audio Analysis](https://developer.spotify.com/documentation/web-api/reference/get-audio-analysis). The only difference is that it doesn't require authentication.

:::caution

Beware, not all tracks have audio data.

:::

```ts
function getAudioData(uri?: string): Promise<any>;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| uri | `string` &#124; `undefined` | URI of the track. If not provided, it will use the current track. |

#### Returns

An object containing the audio data. See the [Spotify Web API reference](https://developer.spotify.com/documentation/web-api/reference/get-audio-analysis) for more details.

#### Example

```ts
// Get audio data from current track
const audioData = await Spicetify.getAudioData();

// Get audio data from a specific track
const audioData = await Spicetify.getAudioData("spotify:track:1qDrWA6lyx8cLECdZE7TV7");
```


=== docs/development/api-wrapper/functions/get-font-style.md ===
---
title: getFontStyle
description: Returns the font style for a given variant.
---

Spicetify provides a function that returns the CSS style for a given font variant used in the Spotify app.

:::tip

This function is used to provide backwards compatibility for older Spicetify extensions and custom apps that use `main-type-` classes.

Instead of using this function to get Spotify stylings, you can simply add the `main-type-<variant>` class to your element.

:::

```ts
function getFontStyle(font: Variant): string;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| font | [`Variant`](/docs/development/api-wrapper/types/variant) | Font variant |

#### Returns

`string` - CSS style for the given font variant.

#### Example

```ts
const style = getFontStyle("forte");

// Returns "viola" if given an invalid variant
// Equivalent to `getFontStyle("viola");`
const style = getFontStyle("invalid-variant");
```


=== docs/development/api-wrapper/functions/remove-from-queue.md ===
---
title: removeFromQueue
description: Removes a track or array of tracks from prioritized queue.
---

:::tip

This works similarly to [`Spicetify.Platform.PlayerAPI.removeFromQueue`](/docs/development/api-wrapper/methods/platform#removefromqueue).

:::

:::caution

If a `uid` is not provided, all tracks with the same `uri` will be removed.

:::

```ts
function removeFromQueue(uri: ContextTrack[]): Promise<void>;
```

#### Parameters

| Name | Type | Description |
| :--- | :--- | :--- |
| `uri` | [`ContextTrack[]`](/docs/development/api-wrapper/types/context-track) | Array of tracks to remove from queue. |

#### Example

```ts
// Remove current track from queue
const currentTrack = Spicetify.Player.data.item;

await Spicetify.removeFromQueue([currentTrack]);

// Remove a track from queue
const trackUri = "spotify:track:4iV5W9uYEdYUVa79Axb7Rh";

await Spicetify.removeFromQueue([ { uri: trackUri } ]);
```


=== docs/development/api-wrapper/functions/show-notification.md ===
---
title: showNotification
description: Show a toast notification inside Spotify.
---

Show a toast notification inside Spotify.

```ts
function showNotification(text: string, isError?: boolean, msTimeout?: number): void;
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| text | `string` | Message to display. Can use inline HTML for styling. |
| isError | `boolean` | If true, toast will be red. Defaults to false. |
| msTimeout | `number` | Time in milliseconds to display the toast. Defaults to Spotify's value. |

#### Example

```ts
// Display a notification
Spicetify.showNotification("My Menu Item clicked!");

// Display a notification with a custom timeout
Spicetify.showNotification("My Menu Item clicked!", false, 1000);

// Display an error notification
Spicetify.showNotification("Something wrong happened", true);

// Display a bolded error notification
Spicetify.showNotification("<b>Something wrong happened</b>", true);
```


=== docs/development/api-wrapper/index.md ===
---
title: API Wrapper
description: 🧰 Everything you need to know about the Spicetify object and API Wrapper.
---

Making an extension from scratch can be a daunting task. Luckily, Spicetify provides a powerful API Wrapper that makes it easy to interact with Spotify's internal APIs as well as provide out-of-the-box methods to help you easily create extensions.

## Spicetify Object
You can access the Spicetify object by typing `Spicetify` in the DevTools console, inside your extension, or `window.top.Spicetify` if you're developing an app inside an `iframe`.

```ts
Spicetify
```

Navigate the sidebar to see all the methods and properties available in the Spicetify object!


=== docs/development/api-wrapper/methods/app-title.md ===
---
title: AppTitle
description: Set of API methods to interact with the Spotify client app title.
---

Spicetify provides a set of API methods to interact with the Spotify client app title.

:::note

These methods only work for the default app title.

:::

```ts
namespace AppTitle {
    function set(title: string): Promise<{ clear: () => void }>;
    function reset(): Promise<void>;
    function get(): Promise<string>;
    function sub(callback: (title: string) => void): { clear: () => void };
}
```

## Methods

### `set`

Set the default app title and force it until canceled. This will override any previous forced title.

:::note

This will temporarily override the current title if a track is being played until the player changes track or the user interacts with the player.

:::

```ts
function set(title: string): Promise<{ clear: () => void }>;
```

#### Parameters

| Name   | Type     | Description |
| ------ | -------- | ----------- |
| `title` | `string` | Title to set |

#### Returns

Promise that resolves to a function to cancel forced title. This doesn't reset the title.

#### Example

```ts
await Spicetify.AppTitle.set("My Extension");
```

### `reset`

Reset app title to default.

```ts
function reset(): Promise<void>;
```

#### Example

```ts
await Spicetify.AppTitle.reset(); // Spotify Premium
```

### `get`

Get current default app title.

:::note

This method cannot get the title of the currently played track.

:::

```ts
function get(): Promise<string>;
```

#### Returns

Current default app title.

#### Example

```ts
const title = await Spicetify.AppTitle.get();
console.log(title); // Spotify Premium
```

### `sub`

Subscribe to title changes.

:::note

This event is not fired when the player changes app title.

:::

```ts
function sub(callback: (title: string) => void): { clear: () => void };
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `callback` | `(title: string) => void` | Callback to call when title changes |

#### Returns

Object with method to unsubscribe.

#### Example

```ts
const { clear } = Spicetify.AppTitle.sub((title) => {
    console.log(title);
});

await Spicetify.AppTitle.set("My Extension"); // Console: My Extension

clear();
```

=== docs/development/api-wrapper/methods/cosmos-async.md ===
---
title: CosmosAsync
description: Asynchronous Cosmos API wrapper used by the Spotify client.
---

Asynchronous Cosmos API wrapper used by the Spotify client. It is used to make requests to the Spotify client's internal API as well as external URLs.

```ts
Spicetify.CosmosAsync
```

It works similarly to `fetch` or `axios` but for each request it will automatically add the required headers and cookies (such as user session token). All responses are parsed as JSON.

:::caution

Be mindful of where you're making a request to, especially if you're making a request to an external URL as it *may* compromise the user's account.

If you're not certain, only use `CosmosAsync` for internal Spotify URLs, or use `fetch` for external URLs.

:::

:::tip

Feel free to reach out to the developers' community on [**Discord**](https://discord.gg/VnevqPp2Rr) if you need help with any of these methods, or if you need a list of all available internal/useful endpoints.

:::

## Methods

```ts
namespace CosmosAsync {
    function head(url: string, headers?: Headers): Promise<Headers>;
    function get(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
    function post(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
    function put(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
    function del(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
    function patch(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
    function sub(url: string, callback: ((b: Response["body"]) => void), onError?: ((e: Error) => void), body?: Body, headers?: Headers): Promise<Response["body"]>;
    function postSub(url: string, body: Body | null, callback: ((b: Response["body"]) => void), onError?: ((e: Error) => void)): Promise<Response["body"]>;
    function request(method: Method, url: string, body?: Body, headers?: Headers): Promise<Response>;
    function resolve(method: Method, url: string, body?: Body, headers?: Headers): Promise<Response>;
}
```

It is worth noting that you can either make a request using the [`request`](#request) method, or use the shorthand methods for each HTTP method.

For example, you can fetch the current client version using either of the following:

```ts
await Spicetify.CosmosAsync.get("sp://desktop/v1/version");
```

or

```ts
await Spicetify.CosmosAsync.request("GET", "sp://desktop/v1/version");
```

For a complete list of available HTTP methods, see [`Method`](/docs/development/api-wrapper/types/cosmos-async/method).

You can also use `CosmosAsync` for Spotify Web API endpoints without having to manually add the required headers and cookies.

```ts
// All endpoints that uses the `sp`, `wg`, and `hm` protocol are internal Spotify endpoints
await Spicetify.CosmosAsync.get("sp://desktop/v1/version");

// Spotify Web API endpoints also works
await Spicetify.CosmosAsync.get("https://api.spotify.com/v1/me");

// Requests to external URLs are NOT safe and may compromise the user's account
// Only use this if you're certain that the URL is safe
// If you need to make a request to an external URL, use `fetch` instead
await fetch("https://example.com");
```

### `head`

Make a `HEAD` request to the specified URL.

```ts
function head(url: string, headers?: Headers): Promise<Headers>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | URL to make the request to. |

### `get`

Make a `GET` request to the specified URL.

```ts
function get(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | URL to make the request to. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `headers` | [`Headers`](/docs/development/api-wrapper/types/cosmos-async/headers) | Request headers. |

Example:

```ts
// Get all playlists in user's library
const res = await Spicetify.CosmosAsync.get("sp://core-playlist/v1/rootlist");
const playlists = res.rows.filter((row) => row.type === "playlist");
```

### `post`

Make a `POST` request to the specified URL.

```ts
function post(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | URL to make the request to. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `headers` | [`Headers`](/docs/development/api-wrapper/types/cosmos-async/headers) | Request headers. |

Example:

```ts
// Skip to the next track in queue
const res = await Spicetify.CosmosAsync.post("sp://player/v2/main/skip_next");
```

### `put`

Make a `PUT` request to the specified URL.

```ts
function put(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | URL to make the request to. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `headers` | [`Headers`](/docs/development/api-wrapper/types/cosmos-async/headers) | Request headers. |

Example:

```ts
// Enable/disable incognito mode
const res = await Spicetify.CosmosAsync.put("sp://scrobble/v1/incognito", { enabled: boolean });
```

### `del`

Make a `DELETE` request to the specified URL.

```ts
function del(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | URL to make the request to. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `headers` | [`Headers`](/docs/development/api-wrapper/types/cosmos-async/headers) | Request headers. |

### `patch`

Make a `PATCH` request to the specified URL.

```ts
function patch(url: string, body?: Body, headers?: Headers): Promise<Response["body"]>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | URL to make the request to. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `headers` | [`Headers`](/docs/development/api-wrapper/types/cosmos-async/headers) | Request headers. |

### `sub`

Make a `SUB` request to the specified URL.

```ts
function sub(url: string, callback: ((b: Response["body"]) => void), onError?: ((e: Error) => void), body?: Body, headers?: Headers): Promise<Response["body"]>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | URL to make the request to. |
| `callback` | `(b: Response["body"]) => void` | Callback function to run when the request is successful. |
| `onError` | `(e: Error) => void` | Callback function to run when the request fails. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `headers` | [`Headers`](/docs/development/api-wrapper/types/cosmos-async/headers) | Request headers. |

### `postSub`

Make a `POST` request to the specified URL, and subscribe to the response.

```ts
function postSub(url: string, body: Body | null, callback: ((b: Response["body"]) => void), onError?: ((e: Error) => void)): Promise<Response["body"]>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | URL to make the request to. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `callback` | `(b: Response["body"]) => void` | Callback function to run when the request is successful. |
| `onError` | `(e: Error) => void` | Callback function to run when the request fails. |

### `request`

Make a request to the specified URL.

```ts
function request(method: Method, url: string, body?: Body, headers?: Headers): Promise<Response>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `method` | [`Method`](/docs/development/api-wrapper/types/cosmos-async/method) | HTTP method to use. |
| `url` | `string` | URL to make the request to. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `headers` | [`Headers`](/docs/development/api-wrapper/types/cosmos-async/headers) | Request headers. |

### `resolve`

Make a request to the specified URL, and resolve the response.

```ts
function resolve(method: Method, url: string, body?: Body, headers?: Headers): Promise<Response>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `method` | [`Method`](/docs/development/api-wrapper/types/cosmos-async/method) | HTTP method to use. |
| `url` | `string` | URL to make the request to. |
| `body` | [`Body`](/docs/development/api-wrapper/types/cosmos-async/body) | Request body. |
| `headers` | [`Headers`](/docs/development/api-wrapper/types/cosmos-async/headers) | Request headers. |


=== docs/development/api-wrapper/methods/graphql.md ===
---
title: GraphQL
description: GraphQL API Wrapper
---

GraphQL API Wrapper used throughout the Spotify client. It is used to communicate with their GraphQL API endpoints for tailored operations such as fetching data, or making mutations.

:::caution

Because this is a private API, it is not documented by Spotify. This documentation is based on the usage of the API in the Spotify client, and may not be accurate.

It is also subject to drastic changes in the future, and has gone through such changes in the past.

Any issues or errors that arise from using this API are not the responsibility of this documentation. If you want to take advantage of this API, bear this in mind.

:::

## Introduction

After reading [`CosmosAsync`](/docs/development/api-wrapper/methods/cosmos-async) and its usage with Spotify [Web API](https://developer.spotify.com/documentation/web-api/), you may be wondering why this is needed, and what is the difference between the two.

In this section, we will be answering these questions:

- What is GraphQL?
- Why do we need to use the GraphQL API wrapper?

### What is GraphQL?

[GraphQL](https://graphql.org/) is a query language for APIs and a runtime for fulfilling those queries with your existing data. It provides a complete and understandable description of the data in your API, gives clients the power to ask for exactly what they need and nothing more, makes it easier to evolve APIs over time, and enables powerful developer tools.

Basically, it is a query language that allows you to query for data in a more flexible way, and allows you to get exactly what you need.

### Why do we need to use the GraphQL API wrapper?

**_"Why can't we just use the Web API?"_**

Unlike the Web API, where we are allowed to get general data for Spotify resources, the GraphQL API is used to get data for tailored operations inside the client, such as fetching colors of the current track for the _Now Playing View_ panel, get all resources for a playlist displayed in the client, or return the search results for a query.

**_"Why can't we use [`CosmosAsync`](/docs/development/api-wrapper/methods/cosmos-async) for GraphQL endpoints?"_**

While you can technically make requests to the GraphQL endpoints using [`CosmosAsync`](/docs/development/api-wrapper/methods/cosmos-async), you would need a lot of special queries to do so, and it would be a lot more complicated than it needs to be.

Take the following endpoint for example, to get the extracted colors used in the _Now Playing View_ panel, you would need to make a request to the following endpoint using [`CosmosAsync`](/docs/development/api-wrapper/methods/cosmos-async):

```ts
await Spicetify.CosmosAsync.get(
  `https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchExtractedColors&variables={"uris":["spotify:image:ab67616d00001e02f16ab998eea7e598a0928ad7"]}&extensions={"persistedQuery":{"version":1,"sha256Hash":"d7696dd106f3c84a1f3ca37225a1de292e66a2d5aced37a66632585eeb3bbbfa"}}`
);
```

compared to using the GraphQL API wrapper:

```ts
const { fetchExtractedColors } = Spicetify.GraphQL.Definitions;
await Spicetify.GraphQL.Request(
  fetchExtractedColors,
  { uris: ['spotify:image:ab67616d00001e02f16ab998eea7e598a0928ad7'] },
);
```

Both would return the same result, but the latter is a lot more readable and easier to use, so why is that? Let's break the endpoint down to understand why.

```js
`https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchExtractedColors&variables={"uris":["spotify:image:ab67616d00001e02f16ab998eea7e598a0928ad7"]}&extensions={"persistedQuery":{"version":1,"sha256Hash":"d7696dd106f3c84a1f3ca37225a1de292e66a2d5aced37a66632585eeb3bbbfa"}}`;
```

Here in the request URL, we can see that there are 3 parts to it:

- `operationName`
- `variables`
- `extensions`

##### `operationName`

This is the name of the operation that you want to perform.

In this case, it is `fetchExtractedColors`.

Nothing too complicated about that.

##### `variables`

Stringified JSON variables that you want to pass to the operation.

In this case, it is `{"uris":["spotify:image:ab67616d00001e02f16ab998eea7e598a0928ad7"]}`.

Not all requests have the same requirements, and without documentation, it would be hard to know which variables are required for each request. Of course, for a specific operation, you can track down the network request using DevTools and see what variables are being passed.

##### `extensions`

Stringified JSON extensions that you want to pass to the operation.

In this case, it is `{"persistedQuery":{"version":1,"sha256Hash":"d7696dd106f3c84a1f3ca37225a1de292e66a2d5aced37a66632585eeb3bbbfa"}}`.

This is the most complicated part of the request. It is used to cache the request, so that the same request does not need to be made again. This is done by passing a `sha256Hash` of the request to the server, and if the server has the same hash, it will return the cached result instead of making the request again.

In this context, the hash is created from the following GraphQL query (as in Spotify `1.2.12`):

```graphql
query fetchExtractedColors($uris: [ID!]!) {
  extractedColors(uris: $uris) {
    __typename
    ... on ExtractedColors {
      colorRaw {
        hex
        isFallback
      }
      colorDark {
        hex
        isFallback
      }
      colorLight {
        hex
        isFallback
      }
    }
    ... on Error {
      message
    }
  }
}
```

By itself, it looks like gibberish, since you have no way of telling what the hash is, and what it is used for. Additionally, as seen from the query above, each request will have their own unique hash, and the request would return an error if the hash is mismatched or not provided.

You also wouldn't be able to tell if Spotify have changed the query server-side until the request breaks, and would need to track down the network request again to get the new hash.

##### Conclusion

As you can see, it is a lot more complicated to make a request to the GraphQL API using [`CosmosAsync`](/docs/development/api-wrapper/methods/cosmos-async), and it would be a lot more complicated to make a request to other GraphQL endpoints as well.

The GraphQL API wrapper simplifies this process by providing a set of definitions used by Spotify themselves, and a method to make requests to the GraphQL API using these definitions.

As long as you have a valid definition, the [`Request`](#request) method will automatically parse the definition and make the request for you.

## Usage

The `GraphQL` object is a set of methods that helps you make requests to the GraphQL API.

```ts
namespace GraphQL {
  const Definitions: Record<Query | string, any>;
  const QueryDefinitions: Record<Query | string, any>;
  const MutationDefinitions: Record<Query | string, any>;
  const ResponseDefinitions: Record<Query | string, any>;
  const Context: Record<string, any>;
  function Request(query: typeof Definitions[Query | string], variables?: Record<string, any>, context?: Record<string, any>): Promise<any>;
  function Handler(context: Record<string, any>): (query: typeof Definitions[Query | string], variables?: Record<string, any>, context?: Record<string, any>) => Promise<any>;
}
```

### `Definitions`

The `Definitions` object is a collection of GraphQL definitions used by Spotify.

You can use these definitions to make requests to the GraphQL API using the [`Request`](#request) method.

```ts
const Definitions: Record<Query | string, any>;
```

For a list of definitions, see [`Query`](/docs/development/api-wrapper/types/graphql/query).

:::caution

The list of methods are subjected to change in the future depending on Spotify's usage.

Also of the nature that this is a private API, the definitions are not documented by Spotify. They can also be removed in the future, both server-side and client-side.

It is recommended to check the definitions in DevTools before using them, and to have several fallbacks in case a definition is removed.

:::

#### Understanding Definitions

:::info

To fully understand these definitions, see [Apollo GraphQL Schema](https://www.apollographql.com/docs/apollo-server/schema/schema/).

In this section, we will only cover the basics of the definitions to get you started as quickly as possible.

:::

Each definition has a similar structure, for this example we will be using a fragment of the `fetchExtractedColors` definition.

```ts
{
    "kind": "Document",
    "definitions": [
        {
            "kind": "OperationDefinition",
            "operation": "query",
            "name": {
                "kind": "Name",
                "value": "fetchExtractedColors"
            },
            "variableDefinitions": [
                {
                    "kind": "VariableDefinition",
                    "variable": {
                        "kind": "Variable",
                        "name": {
                            "kind": "Name",
                            "value": "uris"
                        }
                    },
                    "type": {
                        "kind": "NonNullType",
                        "type": {
                            "kind": "ListType",
                            "type": {
                                "kind": "NonNullType",
                                "type": {
                                    "kind": "NamedType",
                                    "name": {
                                        "kind": "Name",
                                        "value": "ID"
                                    }
                                }
                            }
                        }
                    }
                }
            ],
            "selectionSet": {
                "kind": "SelectionSet",
                "selections": [
                    {
                        // ...
                    }
                ]
            }
        }
    ]
}
```

Inside a `Definition` object, there are 3 main parts:

- `kind`
- `definitions`
- `selectionSet`

For the time being, we will only need to care about `definitions`.

##### `definitions`

There are 2 types of definitions:

- `OperationDefinition`
- `FragmentDefinition`

Essentially, you can only use the `OperationDefinition` type to make requests to the GraphQL API, and `FragmentDefinition` is used to define a fragment that can be used in other definitions.

Each type of definition is filtered into [subsets of the `Definitions` object](#querydefinitions-mutationdefinitions-responsedefinitions) for ease of use.

Inside the `OperationDefinition` object, there are 3 main parts:

- `operation`
- `name`
- `variableDefinitions`

###### `operation`

This is the type of operation that you want to perform, and can be either `query` or `mutation`.

###### `name`

This is the name of the operation that you want to perform. Usually the name of the definition is self-explanatory, so you can be sure of what the operation does by looking at the name.

###### `variableDefinitions`

This is the list of variables that you want to pass to the operation.

This is considered the most important part of the definition, as it is required to make a request to the GraphQL API. We will break it down using the example above in the easiest way possible.

```ts
"variableDefinitions": [
    {
        "kind": "VariableDefinition",
        "variable": {
            "kind": "Variable",
            "name": {
                "kind": "Name",
                "value": "uris"
            }
        },
        "type": {
            "kind": "NonNullType",
            "type": {
                "kind": "ListType",
                "type": {
                    "kind": "NonNullType",
                    "type": {
                        "kind": "NamedType",
                        "name": {
                            "kind": "Name",
                            "value": "ID"
                        }
                    }
                }
            }
        }
    }
]
```

The `variableDefinitions` object is an array of `VariableDefinition` objects. Each object represents a variable that you want to pass to the operation. Each objects can be represented as a property of the `variable` object passed onto the `Request` method.

In this case, the `variableDefinitions` object only has 1 `VariableDefinition` object named `uris`, so we only need to pass 1 variable to the `Request` method.

Inside the `VariableDefinition` object, there are 2 main parts:

- `variable`
- `type`

###### `variable`

This is the name of the variable that you want to pass to the operation. We only need to care about the name of the variable, as the rest of the object is used to parse the definition.

In the example above, the name of the variable is `uris`, so in the [`Request`](#request) method, we would pass the variable as follows:

```ts
Spicetify.GraphQL.Request(
  Spicetify.GraphQL.Definitions.fetchExtractedColors,
  { uris: /* ... */ },
);
```

###### `type`

Each type has its equivalent type in JavaScript, and in the example above we get the following reference:

- `NonNullType`: `!`, meaning that the variable is required.
- `ListType`: `[]`, meaning that the variable is an array.
- `NamedType`: `string`, meaning that the variable is a string.


:::note

Sometimes a variable is not listed as `NonNullType`, but is still required to make a successful request. This is because the variable is required by the operation itself, and not the definition.

:::

:::info

For a full list of types, see [GraphQL Scalar Types](https://graphql.org/learn/schema/#scalar-types).

:::

As we can see, the variable `uris` is a required array of strings.

To conclude, the `fetchExtractedColors` method requires:

  1. A parameter named `uris`
  2. The `uris` parameter is a required array of strings

So the request would look like this:

```ts
Spicetify.GraphQL.Request(
  Spicetify.GraphQL.Definitions.fetchExtractedColors,
  { uris: ['spotify:image:ab67616d00001e02f16ab998eea7e598a0928ad7'] },
);
```

Using the same logic, you can parse the rest of the definitions to know the structure of the response by understanding the `selectionSet` object, or explore the response in DevTools.

### `QueryDefinitions`, `MutationDefinitions`, `ResponseDefinitions`

The `QueryDefinitions`, `MutationDefinitions` and `ResponseDefinitions` objects are subsets of the `Definitions` object.

These are filtered by their type, and are used to make different types of requests to the GraphQL API using the [`Request`](#request) method.

```ts
const QueryDefinitions: Record<Query | string, any>;
const MutationDefinitions: Record<Query | string, any>;
const ResponseDefinitions: Record<Query | string, any>;
```

Each type of definition is used for a different purpose:

- `QueryDefinitions`: Used to make GraphQL queries. These are queries that are used to fetch data. (e.g. fetching playlist metadata)
- `MutationDefinitions`: Used to make GraphQL mutations. These are queries that are used to mutate data. (e.g. adding a track to a playlist)
- `ResponseDefinitions`: Used to parse GraphQL responses.

Most of the time, you would only need to use the `QueryDefinitions` and `MutationDefinitions` subsets to make requests to the GraphQL API.

### `Request`

The `Request` method is used to make requests to the GraphQL API using the definitions provided.

```ts
function Request(
  query: typeof Definitions[Query | string],
  variables?: Record<string, any>,
  context?: Record<string, any>
): Promise<any>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `query` | `Record<string, any>` | The definition to use for the request. |
| `variables` | `Record<string, any>` | The variables to pass to the operation. |
| `context` | `Record<string, any>` | The context to use for the request. |

| Return | Type | Description |
| --- | --- | --- |
| `Promise` | `any` | The response from the request. |

#### `query`

The `query` parameter is the definition to use for the request. It can be any definition from the [`Definitions`](#definitions) object.

#### `variables`

The `variables` parameter is the variables to pass to the operation. It can be any valid variables for the operation.

#### `context`

The `context` parameter is the context to use for the request. It can be any valid context for the operation. These will be passed onto the request as parameters.

##### Example

```ts
// Get all data displayed in an album page
const { getAlbum } = Spicetify.GraphQL.Definitions;
await Spicetify.GraphQL.Request(
  getAlbum,
  { uri: "spotify:album:3lvY0ag1k3qiPCsEM3Wnku", locale: "en", limit: 50, offset: 0 },
  { persistCache: true },
);
```

### `Context`

The `Context` class is used to set the context for the [`Handler`](#handler) method.

```ts
const Context: Record<string, any>;
```

On Spotify `1.2.12`, the class contains the following properties:

| Property | Type | Description |
| --- | --- | --- |
| `accessToken` | `string` | The access token to use for the request. |
| `globalRequestHeaders` | `string[][]` | The global request headers to use for the request. |
| `locale` | `string` | The locale to use for the request. |
| `market` | `string` | The market to use for the request. |

### `Handler`

The `Handler` method is used to create a handler for the [`Request`](#request) method.

```ts
function Handler(
  context: Record<string, any>
): (
  query: typeof Definitions[Query | string],
  variables?: Record<string, any>,
  context?: Record<string, any>
) => Promise<any>;
```

| Parameter | Type | Description |
| --- | --- | --- |
| `context` | `Record<string, any>` | The context to use for the request. |

:::info

The `context` parameter is the same as the [`Context`](#context) class.

:::

| Return | Type | Description |
| --- | --- | --- |
| `Function` | `(query: Record<string, any>, variables?: Record<string, any>, context?: Record<string, any>) => Promise<any>` | The handler for the request. |

:::info

The [`Request`](#request) method is a preinitialized version of the [`Handler`](#handler) method using the current context.

:::

=== docs/development/api-wrapper/methods/keyboard.md ===
---
title: Keyboard
description: A wrapper for keyboard shortcuts.
---

Spicetify provides its own method for global keyboard shortcuts. You can specify actions for your extension when the user presses a keyboard shortcut.

:::tip

`Spicetify.Keyboard` is a wrapper of [`Spicetify.Mousetrap`](/docs/development/api-wrapper/modules#mousetrap) configured to be compatible with legacy Spotify.

New extensions are advised to use the module instead.

:::

:::caution

All shortcuts registered by `Spicetify.Keyboard` are global. Be mindful of conflicts with other extensions or the Spotify client itself.

:::

```ts
namespace Keyboard {
    const KEYS: Record<ValidKey, string>;
    function registerShortcut(keys: KeysDefine, callback: (event: KeyboardEvent) => void): void;
    function _deregisterShortcut(keys: KeysDefine): void;
    function changeShortcut(keys: KeysDefine, newKeys: KeysDefine): void;
};
```

## Properties

### `KEYS`

An object containing a list of valid keys, mapped to their valid names.

```ts
const { KEYS } = Spicetify.Keyboard;
```

Refer to [this table](/docs/development/api-wrapper/types/keyboard/validkey) for a list of valid keys.

#### Example

```ts
const { KEYS } = Spicetify.Keyboard;
console.log(KEYS["CAPS"]); // "Capslock"
```

## Methods

### `registerShortcut`

Register a global keyboard shortcut.

```ts
function registerShortcut(keys: KeysDefine, callback: (event: KeyboardEvent) => void): void;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| keys | [`KeysDefine`](/docs/development/api-wrapper/types/keyboard/keysdefine) | Keyboard shortcut to register. |
| callback | `(event: KeyboardEvent) => void` | Callback function to run when the shortcut is triggered. |

#### Example

```ts
// Equivalent to `Spicetify.Keyboard.registerShortcut({ key: "p", ctrl: true, shift: true }, (event) => { ... })`
Spicetify.Keyboard.registerShortcut("ctrl+shift+p", (event) => {
    // Do something with the event
    Spicetify.showNotification("Shortcut triggered!");
});
```

### `_deregisterShortcut`

Deregister a global keyboard shortcut.

```ts
function _deregisterShortcut(keys: KeysDefine): void;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| keys | [`KeysDefine`](/docs/development/api-wrapper/types/keyboard/keysdefine) | Keyboard shortcut to deregister. |

#### Example

```ts
Spicetify.Keyboard._deregisterShortcut("ctrl+shift+p");
```

### `changeShortcut`

Change a global keyboard shortcut to a new shortcut while keeping the callback.

```ts
function changeShortcut(keys: KeysDefine, newKeys: KeysDefine): void;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| keys | [`KeysDefine`](/docs/development/api-wrapper/types/keyboard/keysdefine) | Keyboard shortcut to change. |
| newKeys | [`KeysDefine`](/docs/development/api-wrapper/types/keyboard/keysdefine) | New keyboard shortcut to change to. |

#### Example

```ts
Spicetify.Keyboard.changeShortcut("ctrl+shift+p", "ctrl+shift+o");
```


=== docs/development/api-wrapper/methods/local-storage.md ===
---
title: LocalStorage
description: Get and set data in local storage.
---

Spicetify provides a wrapper for `localStorage` to make it easier to use.

:::tip

All keys created via this method are generic and stored as-is.

If you wish to store values that are specific for each user account, you can use [`Platform.LocalStorageAPI`](/docs/development/api-wrapper/methods/platform#localstorageapi) instead.

:::

```ts
namespace LocalStorage {
    function clear(): void;
    function get(key: string): string | null;
    function remove(key: string): void;
    function set(key: string, value: string): void;
};
```

## Methods

### `clear`

Empties the list associated with the object of all key/value pairs, if there are any.

:::warning

This method will remove all data in local storage, not just the data that Spicetify uses. This essentially resets the client to its default state.

It will also wipe all data stored by extensions and custom apps (e.g. Marketplace, Lyrics Plus, etc.)

:::

```ts
clear(): void
```

### `get`

Get key value from local storage.

```ts
get(key: string): string | null
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| key | `string` | Key to get value from. |

#### Example

```ts
const value = Spicetify.LocalStorage.get("foo");
```

### `remove`

Delete key from local storage.

```ts
remove(key: string): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| key | `string` | Key to delete. |

#### Example

```ts
Spicetify.LocalStorage.remove("foo");
```

### `set`

Set new value for key in local storage.

```ts
set(key: string, value: string): void
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| key | `string` | Key to set value for. |
| value | `string` | Value to set. |

#### Example

```ts
Spicetify.LocalStorage.set("foo", "bar");
```


=== docs/development/api-wrapper/methods/panel.md ===
---
title: Panel
description: API wrapper to interact with Spotify's panel/right sidebar.
---

Spicetify provides a wrapper for the Spotify `PanelAPI` method to make it easier to use, as well as providing a method to help you register your own panel.

```ts
namespace Panel {
  const reservedPanelIds: Record<string | number, string | number>;
  const Components: {
    PanelSkeleton: any;
    PanelContent: any;
    PanelHeader: any;
  };
  function hasPanel(id: number): boolean;
  function getPanel(id: number): React.ReactNode | undefined;
  function setPanel(id: number): Promise<void>;
  function subPanelState(callback: (id: number) => void): void;
  function registerPanel(props: PanelProps): {
    id: number;
    toggle: () => Promise<void>;
    onStateChange: (callback: (isActive: boolean) => void) => void;
    isActive: boolean;
  };
  const currentPanel: number;
}
```

Internally, Spotify uses an enum to cross-reference the IDs of the panels with the panels themselves. As such, all methods in `Panel` will only accept IDs that are reserved by Spotify and those that are manually registered via `registerPanel`.

## Properties

### `reservedPanelIds`

An object of reserved panel IDs used by Spotify.

```ts
const reservedPanelIds: Record<string | number, string | number>;
```

This list is dynamic, so it is recommended to check the list in the console.

As of Spotify `1.2.12`, there are currently 5 reserved IDs:

```ts
{
    0: "Disabled",
    1: "BuddyFeed",
    2: "NowPlayingView",
    3: "WhatsNewFeed",
    4: "Puffin",
}
```

### `Components`

Collection of React Components used by Spotify in the Panel.

Refer to [Registering a custom Panel](#registering-a-custom-panel) for more information on how to use these components.

```ts
const Components: {
  PanelSkeleton: any;
  PanelContent: any;
  PanelHeader: any;
};
```

### `currentPanel`

ID of the current Panel.

```ts
const currentPanel: number;
```

Example:

```ts
console.log(Spicetify.Panel.currentPanel); // 0
```

## Methods

### `hasPanel`

Check whether or not a Panel with the provided ID is registered.

```ts
function hasPanel(id: number): boolean;
```

Example:

```ts
Spicetify.Panel.hasPanel(0); // true, reserved to "Disabled" by Spotify
Spicetify.Panel.hasPanel('Disabled'); // false, ID is not a number
Spicetify.Panel.hasPanel(5); // false, ID is not registered
```

### `getPanel`

Get the Panel with the provided ID.

```ts
function getPanel(id: number): React.ReactNode | string | undefined;
```

Example:

```ts
Spicetify.Panel.getPanel(0); // "Disabled"
Spicetify.Panel.getPanel('Disabled'); // undefined, ID is not a number
Spicetify.Panel.getPanel(5); // undefined, ID is not registered
```

### `setPanel`

Set the Panel state with the provided ID.

```ts
function setPanel(id: number): Promise<void>;
```

:::caution

On older Spotify versions (around `1.2.6`), the `PanelAPI` method is incomplete, most notably the `getPanel` method.

Spicetify offers a workaround for this by storing its own state of the current Panel ID, however this state is only updated after Spotify internal components are rendered due to how the `PanelAPI` method works on older versions.

If you decide to interact directly with the `PanelAPI` method instead of using the wrapper, the `render` method will not work as expected.

```ts
// Normal way of using the Panel method, will work as expected
Spicetify.Panel.setPanel(0);

// Interacting directly with the PanelAPI method will cause some abnormalities
// as state is updated after Spotify internal components are rendered
Spicetify.Platform.PanelAPI.setPanel(0);
```

:::

Example:

```ts
await Spicetify.Panel.setPanel(0); // Close the Panel

await Spicetify.Panel.setPanel(1); // Open BuddyFeed

// Open the Panel with ID 5
// If the ID is not registered, it will be set to 0
await Spicetify.Panel.setPanel(5);
```

### `subPanelState`

Subscribe to Panel changes.

```ts
function subPanelState(callback: (id: number) => void): void;
```

Example:

```ts
Spicetify.Panel.subPanelState((id) => {
  console.log(id); // 2
  // Do something
});
```

### `registerPanel`

Register a new Panel and return its methods and properties.

:::note

To avoid conflict with Spotify and other extensions on current version and future updates, an ID will be automatically assigned to the Panel.

:::

:::tip

To make it easier and convenient for developers to use the Panel API, this method by default wraps the children passed into a Panel skeleton and content wrapper.

However, this also means that the props passed onto the Panel cannot be updated after the Panel is registered. (most notably the `label` and `header` props)

If you want to customize the Panel, look into [Registering a custom Panel](#registering-a-custom-panel).

:::

```ts
function registerPanel(props: PanelProps): {
  id: number;
  toggle: () => Promise<void>;
  onStateChange: (callback: (isActive: boolean) => void) => void;
  isActive: boolean;
};
```

#### Parameters

| Name    | Type                                                                | Description             |
| ------- | ------------------------------------------------------------------- | ----------------------- |
| `props` | [PanelProps](/docs/development/api-wrapper/types/panel/panel-props) | Properties of the Panel |

Example:

```ts
const { id, toggle, onStateChange, isActive } = Spicetify.Panel.registerPanel({
  label: 'My Panel',
  children: <div>My Panel</div>,
});

console.log(id); // 5
console.log(isActive); // false

await toggle(); // Open the Panel

console.log(isActive); // true

onStateChange((isActive) => {
  console.log(isActive); // false
});
```

:::tip

`onStateChange` callback function will be called immediately after registering a callback due to how the `PanelAPI` method works.

:::

#### Registering a custom Panel

Ideally, you would not need to use these components directly, but in case the `registerPanel` method does not satisfy your needs, you can use these along with the `isCustom` prop to create your own customised panel.

#### Props

See [`PanelHeaderProps`](/docs/development/api-wrapper/types/react-component/panel-header-props), [`PanelContentProps`](/docs/development/api-wrapper/types/react-component/panel-content-props), and [`PanelSkeletonProps`](/docs/development/api-wrapper/types/react-component/panel-skeleton-props).

#### Example

```tsx
const PanelAction = () => {
    return (
        <Spicetify.ReactComponent.TooltipWrapper label="Show notification">
            <button onClick={() => Spicetify.showNotification('Hello World')}>
                <Spicetify.ReactComponent.IconComponent
                    semanticColor="textBase"
                    dangerouslySetInnerHTML={{ __html: Spicetify.SVGIcons["play"] }}
                    iconSize={16}
                />
            </button>
        </Spicetify.ReactComponent.TooltipWrapper>
    )
}

// Ideally, you would want to memoize this component
// to prevent unnecessary re-renders if you were to pass props

// Props will have a single `panel` property for the panel ID
const Panel = ({ panel }) => {
    // For example, if you want to change the header title
    const [title, setTitle] = React.useState('Hello World');
    // Or if you want to display extra actions on the header in a stateful manner
    const [showActions, setShowActions] = React.useState(false);

    return (
        <Spicetify.ReactComponent.PanelSkeleton
            label="Hello World"
            style={{
                "--panel-width": "300px"
            }}
        >
            <Spicetify.ReactComponent.PanelContent className="my-panel-content">
                <Spicetify.ReactComponent.PanelHeader
                    title={title}
                    link="/collection" // Can be an URI or an external URL
                    panel={panel}
                    actions={showActions && <PanelAction />}
                    onClose={() => Spicetify.showNotification('Closed')}
                    onBack={() => Spicetify.showNotification('Back')}
                />
                <div className="my-panel-body">
                    Hello World
                </div>
                <button onClick={() => setTitle('Hello World 2')}>Change title</button>
                <button onClick={() => setShowActions(!showActions)}>Toggle actions</button>
            </Spicetify.ReactComponent.PanelContent>
        </Spicetify.ReactComponent.PanelSkeleton>
    )
};

// Finally, register the Panel
const { id, toggle, onStateChange, isActive } = Spicetify.Panel.registerPanel({
    children: <Panel />,
    isCustom: true
});
```


=== docs/development/api-wrapper/methods/platform.md ===
---
title: Platform
description: A vast collection of internal APIs used throughout the Spotify client.
---

Spicetify provides a vast collection of internal APIs used throughout the Spotify client. These APIs are used to interact with the Spotify client and modify its behavior.

Explore all available methods in DevTools:

```js
Spicetify.Platform
```

:::warning

Because these APIs are internal, they are not guaranteed to be stable and may *drastically* change in the future. Use them at your own risk.

:::

## Usage

Since these APIs differ between each version of the Spotify client, we cannot provide a complete list of all available APIs. Instead, we provide a list of APIs that may prove useful for extension developers and that have generally not changed much over the years.

:::note

If a method is not listed here, it could be that:
  - It is objectively complicated to use.
  - It is not well documented.
  - It is generally not useful/convenient for extension developers.
  - It is not stable and may change/has changed drastically throughout versions.

:::

:::tip

Feel free to reach out to the developers' community on [**Discord**](https://discord.gg/VnevqPp2Rr) if you need help with any of these methods.

:::

### ClipboardAPI

The Spotify client doesn't allow users to copy directly from the client (say `Ctrl + C`), but it does provide an API to copy text to the clipboard.

```ts
interface ClipboardAPI {
    copy: (text: string) => Promise<void>;
    paste: () => Promise<string | undefined>;
};
```

#### `copy`

Copy text to clipboard.

:::info

The parameter passed will be stringified before being copied to the clipboard.

:::

| Parameter | Type | Description |
| --- | --- | --- |
| `text` | `string` | Text to copy. |

Example:

```ts
// Will be copied as "Hello World!"
await Spicetify.Platform.ClipboardAPI.copy("Hello World!");

// Will be stringified to '{"0":0,"1":0,"2":0,"3":0}'
await Spicetify.Platform.ClipboardAPI.copy(new Uint16Array(4));
```

#### `paste`

Paste text from clipboard. Returns a string.

:::caution

If the content of the clipboard is not a string, this method will return `undefined`.

:::

Example:

```ts
await Spicetify.Platform.ClipboardAPI.copy("Hello World!");
await Spicetify.Platform.ClipboardAPI.paste(); // "Hello World!"

await Spicetify.Platform.ClipboardAPI.copy(new Uint16Array(4));
await Spicetify.Platform.ClipboardAPI.paste(); // '{"0":0,"1":0,"2":0,"3":0}'
```

### History

Spotify has their own router API that allows you to navigate to different pages within the client. You can use it to navigate within your custom apps or push new pages to the history stack and display them for the users.

```ts
interface History {
    push: (path: Location | string) => void;
    replace: (path: Location | string) => void;
    goBack: () => void;
    goForward: () => void;
    listen: (listener: (location: Location) => void) => () => void;
    entries: Location[];
    location: Location;
};
```
Their `Location` object is a simple object that contains the `pathname` of the current page as well as the relevant query parameters and state. It looks roughly like this:

```ts
interface Location {
    pathname: string;
    search: string;
    hash: string;
    state: Record<string, any>;
};
```

#### `push`

Push a new location to the history stack.

You can pass either a `Location` object or a `pathname` string to this method.

| Parameter | Type | Description |
| --- | --- | --- |
| `path` | `Location` &#124; `string` | Location to push. |

```ts
Spicetify.Platform.History.push("/app/your-app");

Spicetify.Platform.History.push({
    pathname: "/app/your-app",
    search: "?foo=bar",
    hash: "#baz",
    state: { foo: "bar" },
});
```

#### `replace`

Replace the current location in the history stack.

:::caution

Users will **not** be able to go back to the previous page.

:::

You can pass either a `Location` object or a `pathname` string to this method.

| Parameter | Type | Description |
| --- | --- | --- |
| `path` | `Location` &#124; `string` | Location to replace. |

```ts
// Replace the current location with a new one.
Spicetify.Platform.History.replace("/app/your-app");

Spicetify.Platform.History.replace({
    pathname: "/app/your-app",
    search: "?foo=bar",
    hash: "#baz",
    state: { foo: "bar" },
});
```

#### `goBack`

Go back to the previous location in the history stack.

#### `goForward`

Go forward to the next location in the history stack.

:::caution

The page may not be fully loaded when this event is fired. You may need to wait for the DOM to finish loading before you can interact with it.

:::

#### `listen`

Listen to changes in the history stack. Fires whenever the user navigates to a new page.

| Parameter | Type | Description |
| --- | --- | --- |
| `listener` | `(location: Location) => void` | Callback to fire when the user navigates to a new page. |

Example:

```ts
Spicetify.Platform.History.listen((location) => {
    // Log the current pathname every time the user navigates to a new page.
    console.log(location.pathname);
});
```

#### `entries`

An array of all locations in the history stack.

#### `location`

The current location in the history stack.

### LocalStorageAPI

Spotify provides a simple API to interact with the browser's local storage. All keys are stored using the current user's username as the namespace.

Inside `localStorage`, the keys are stored using the following format:

```ts
`${namespace}:${key}`
```

:::tip

All keys created using this method will be namespaced using the current user's username.

If you wish to create a generic key, you can use [`Spicetify.LocalStorage`](./local-storage) instead.

:::

```ts
interface LocalStorageAPI {
    items: Record<string, any>;
    namespace: string;
    getItem: (key: string) => any;
    setItem: (key: string, value: any) => void;
    clearItem: (key: string) => void;
};
```

#### `items`

An object containing all the keys and values stored in the local storage.

All keys in `items` are stored using the aforementioned format, with it's value pair being the value stored in the local storage parsed using `JSON.parse`.

Example:

```ts
Spicetify.Platform.LocalStorageAPI.items; // { "username:foo": "bar" }
```

#### `namespace`

The namespace used to store all keys in the local storage. Usually the current user's username.

#### `getItem`

Get a value from the local storage. Returns a parsed value using `JSON.parse`.

| Parameter | Type | Description |
| --- | --- | --- |
| `key` | `string` | Key to get. |

Example:

```ts
// This is equivalent to Spicetify.Platform.LocalStorageAPI.items["username:foo"]
// or localStorage.getItem("username:foo")
Spicetify.Platform.LocalStorageAPI.getItem("foo"); // "bar"
```

#### `setItem`

Set a value in the local storage. The value will be stringified using `JSON.stringify`.

| Parameter | Type | Description |
| --- | --- | --- |
| `key` | `string` | Key to set. |
| `value` | `any` | Value to set. Can be any type. |

Example:

```ts
Spicetify.Platform.LocalStorageAPI.setItem("foo", { bar: "baz" });
// localStorage.getItem("username:foo") === '{"bar":"baz"}'
```

#### `clearItem`

Clear a value from the local storage.

| Parameter | Type | Description |
| --- | --- | --- |
| `key` | `string` | Key to clear. |

Example:

```ts
Spicetify.Platform.LocalStorageAPI.clearItem("foo");
// localStorage.getItem("username:foo") === null
```

### PlatformData

Contains data about the current platform, such as the current Spotify client version, operating system, and more.

```ts
interface PlatformData {
    app_platform: string;
    client_capabilities: Record<string, any>;
    client_version_triple: string;
    client_version_quadruple: string;
    client_version_quintuple: string;
    event_sender_context_information: Record<string, any>;
    os_name: string;
    os_version: string;
}
```

#### `app_platform`

The current Spotify client platform.

Example:

```ts
Spicetify.Platform.PlatformData.app_platform; // "win32"
```

#### `client_capabilities`

An object containing the current client capabilities. This usually contains information relating to functionality inside the Spotify client, such as whether or not the client can autostart.

#### `client_version_triple`, `client_version_quadruple`, `client_version_quintuple`

The current Spotify client version. Usually in the format `1.2.8`, `1.2.8.923`, or `1.2.8.923.g4f94bf0d`.

#### `event_sender_context_information`

An object containing information about the current operating system. Used for analytics throughout the Spotify client.

Example:

```ts
Spicetify.Platform.PlatformData.event_sender_context_information; // { "platform_type:"windows", "os_version": "10.0.19042" }
```

This could also help you diagnose issues with your custom apps. For example, if you're using a custom app on Windows and you're getting an error, you can check the `event_sender_context_information` object to see if the `platform_type` is `windows` or `macos`.

#### `os_name`

The current operating system.

Example:

```ts
Spicetify.Platform.PlatformData.os_name; // "windows"
```

#### `os_version`

The current operating system version.

Example:

```ts
Spicetify.Platform.PlatformData.os_version; // "10.0.19042"
```

### Session

Contains data about the current user session, such as the current user's access token, locale, and more.

```ts
interface Session {
    accessToken: string;
    accessTokenExpirationTimestampMs: number;
    locale: string;
}
```

:::note

If you're trying to make an API request to internal Spotify endpoints, you should use [`CosmosAsync`](/docs/development/api-wrapper/methods/cosmos-async)

:::

#### `accessToken`

The current user's access token. This is used to authenticate requests to the Spotify API.

#### `accessTokenExpirationTimestampMs`

The timestamp in milliseconds when the current user's access token expires.

#### `locale`

The current user's locale.

Example:

```ts
Spicetify.Platform.Session.locale; // "en"
```

### Translations

Contains translation strings used throughout the current Spotify client.

```ts
interface Translations {
    [key: string]: string;
}
```

### PlayerAPI

Contains methods to interact with the Spotify client's player.

:::tip

It is recommended to use [`Player`](/docs/development/api-wrapper/methods/player) instead for ease of use, or access `Player.origin` to get the `PlayerAPI` object.

:::

```ts
interface PlayerAPI {
    addToQueue: (items: ContextTrack[]) => Promise<void>;
    clearQueue: () => Promise<void>;
    pause: () => Promise<void>;
    play: (uri: ContextTrack, context, options = {}) => Promise<void>;
    removeFromQueue: (items: ContextTrack[]) => Promise<void>;
    resume: () => Promise<void>;
    seekBackward: (ms: number) => Promise<void>;
    seekBy: (ms: number) => Promise<void>;
    seekForward: (ms: number) => Promise<void>;
    seekTo: (ms: number) => Promise<void>;
    setRepeat: (mode: RepeatMode) => Promise<void>;
    setShuffle: (shuffle: boolean) => Promise<void>;
    setSpeed: (speed: number) => Promise<void>;
}
```

#### `RepeatMode`

Enum for the repeat mode.

```ts
enum RepeatMode {
    Off = 0,
    RepeatAll = 1,
    RepeatOne = 2,
}
```

#### `addToQueue`

Add items to the current user's queue.

:::tip

This works as if the user had clicked the `Add to queue` button in the Spotify client, meaning a notification will be shown.

If you want to add items to the queue without showing a notification, use [`Spicetify.addToQueue`](/docs/development/api-wrapper/functions/add-to-queue).

:::

```ts
await Spicetify.Platform.PlayerAPI.addToQueue(items);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `items` | [`ContextTrack[]`](/docs/development/api-wrapper/types/context-track) | Items to add to the queue. |

##### Example

```ts
// Add a track to the queue

// 505 - Arctic Monkeys
const track = { uri: "spotify:track:0BxE4FqsDD1Ot4YuBXwAPp" };

await Spicetify.Platform.PlayerAPI.addToQueue([track]);
```

#### `clearQueue`

Clear the current user's queue.

```ts
await Spicetify.Platform.PlayerAPI.clearQueue();
```

#### `pause`

Pause the current user's playback.

```ts
await Spicetify.Platform.PlayerAPI.pause();
```

#### `play`

:::tip

It is recommended to use [`Player.playUri`](/docs/development/api-wrapper/methods/player#playuri) instead for ease of use.

:::

Start playback of a track.

```ts
await Spicetify.Platform.PlayerAPI.play(uri, context, options);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `uri` | [`ContextTrack`](/docs/development/api-wrapper/types/context-track) | The track to play. |
| `context` | `Record<string, any>` | The context of the track. Must be an object. |
| `options` | `Record<string, any>` &#124; `undefined` | Playback options. |

##### Example

```ts
// 505 - Arctic Monkeys
const track = { uri: "spotify:track:0BxE4FqsDD1Ot4YuBXwAPp" };

// Play the track
// Spicetify.Player.playUri(track.uri);
await Spicetify.Platform.PlayerAPI.play(track, {}, {});
```

#### `removeFromQueue`

Remove items from the current user's queue.

:::tip

This works similarly to [`Spicetify.removeFromQueue`](/docs/development/api-wrapper/functions/remove-from-queue).

:::

```ts
await Spicetify.Platform.PlayerAPI.removeFromQueue(items);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `items` | [`ContextTrack[]`](/docs/development/api-wrapper/types/context-track) | Items to remove from the queue. |

##### Example

```ts
// Remove a track from the queue

// 505 - Arctic Monkeys
const track = { uri: "spotify:track:0BxE4FqsDD1Ot4YuBXwAPp" };

// Remove the track if it's in the queue
await Spicetify.Platform.PlayerAPI.removeFromQueue([track]);
```

#### `resume`

Resume the current user's playback.

```ts
await Spicetify.Platform.PlayerAPI.resume();
```

#### `seekBackward`

Seek backward in the current user's playback.

```ts
await Spicetify.Platform.PlayerAPI.seekBackward(ms);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `ms` | `number` | The number of milliseconds to seek backward. |

##### Example

```ts
// Seek backward 10 seconds
await Spicetify.Platform.PlayerAPI.seekBackward(10000);
```

#### `seekBy`

Seek by a number of milliseconds in the current user's playback.

If passed a negative number, it will seek backward.

```ts
await Spicetify.Platform.PlayerAPI.seekBy(ms);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `ms` | `number` | The number of milliseconds to seek by. |

##### Example

```ts
// Seek forward 10 seconds
await Spicetify.Platform.PlayerAPI.seekBy(10000);

// Seek backward 10 seconds
await Spicetify.Platform.PlayerAPI.seekBy(-10000);
```

#### `seekForward`

Seek forward in the current user's playback.

```ts
await Spicetify.Platform.PlayerAPI.seekForward(ms);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `ms` | `number` | The number of milliseconds to seek forward. |

##### Example

```ts
// Seek forward 10 seconds
await Spicetify.Platform.PlayerAPI.seekForward(10000);
```

#### `seekTo`

Seek to a specific position in the current user's playback.

:::tip

[`Player.seek`](/docs/development/api-wrapper/methods/player#seek) support both seeking by a number of milliseconds and by a percentage.

:::

```ts
await Spicetify.Platform.PlayerAPI.seekTo(ms);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `ms` | `number` | The position in milliseconds to seek to. |

##### Example

```ts
// Seek to 1 minute
await Spicetify.Platform.PlayerAPI.seekTo(60000);
```

#### `setRepeat`

Set the current user's repeat mode.

```ts
await Spicetify.Platform.PlayerAPI.setRepeat(mode);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `mode` | [`RepeatMode`](#repeatmode) | The repeat mode to set. |

##### Example

```ts
// Set repeat mode to repeat one
await Spicetify.Platform.PlayerAPI.setRepeat(2);
```

#### `setShuffle`

Set the current user's shuffle mode.

```ts
await Spicetify.Platform.PlayerAPI.setShuffle(shuffle);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `shuffle` | `boolean` | Whether to enable shuffle mode. |

##### Example

```ts
// Enable shuffle mode
await Spicetify.Platform.PlayerAPI.setShuffle(true);
```

#### `setSpeed`

Set the current user's playback speed.

:::note

This only works for podcasts. Music playback speed is unaffected.

:::

```ts
await Spicetify.Platform.PlayerAPI.setSpeed(speed);
```

| Parameter | Type | Description |
| --- | --- | --- |
| `speed` | `number` | The playback speed to set. |

##### Example

```ts
// Set playback speed to 1.5x
await Spicetify.Platform.PlayerAPI.setSpeed(1.5);
```


=== docs/development/api-wrapper/methods/player.md ===
---
title: Player
description: A collection of methods to interact with the Spotify player.
---

Spicetify provides a collection of methods to interact with the Spotify player. You can get the current player state, play/pause, skip to next/previous track, set repeat/shuffle mode, and more.

This is mostly a wrapper of the `Spicetify.Platform.PlayerAPI` object.

```ts
namespace Player {
    function addEventListener(type: string, callback: (event?: Event) => void): void;
    function addEventListener(type: "songchange", callback: (event?: Event & { data: PlayerState }) => void): void;
    function addEventListener(type: "onplaypause", callback: (event?: Event & { data: PlayerState }) => void): void;
    function addEventListener(type: "onprogress", callback: (event?: Event & { data: number }) => void): void;
    function back(): void;
    const data?: PlayerState;
    function decreaseVolume(): void;
    function dispatchEvent(event: Event): void;
    const eventListeners: {
        [key: string]: Array<(event?: Event) => void>
    };
    function formatTime(milisecond: number): string;
    function getDuration(): number;
    function getMute(): boolean;
    function getProgress(): number;
    function getProgressPercent(): number;
    function getRepeat(): number;
    function getShuffle(): boolean;
    function getHeart(): boolean;
    function getVolume(): number;
    function increaseVolume(): void;
    function isPlaying(): boolean;
    function next(): void;
    function pause(): void;
    function play(): void;
    function playUri(uri: string, context?: any, options?: any): Promise<void>;
    function removeEventListener(type: string, callback: (event?: Event) => void): void;
    function seek(position: number): void;
    function setHeart(status: boolean): void;
    function setMute(state: boolean): void;
    function setRepeat(mode: number): void;
    function setShuffle(state: boolean): void;
    function setVolume(level: number): void;
    function skipBack(amount?: number): void;
    function skipForward(amount?: number): void;
    function toggleHeart(): void;
    function toggleMute(): void;
    function togglePlay(): void;
    function toggleRepeat(): void;
    function toggleShuffle(): void;
}
```

## Properties

### data

An object contains all information about current track and player.

:::caution

If the current player doesn't have any track, `data` will be `null`. Always check for `null` before using `data` to avoid errors.

:::

```ts
Spicetify.Player.data;
```

#### Return

[`PlayerState`](/docs/development/api-wrapper/types/player-state.md)


#### Example

```ts
// Get current track URI
const currentURI = Spicetify.Player.data?.item.uri;
if (currentURI) {
    console.log(currentURI);
}
```

### eventListeners

An object containing all registered event listeners.

```ts
Spicetify.Player.eventListeners;
```

#### Return

```ts
{
    [key: string]: Array<(event?: Event) => void>
}
```

| Key | Description |
| --- | ----------- |
| `key` | Event type |
| `value` | Array of registered event listeners |

## Methods

### addEventListener

Register a listener of `type` on Spicetify.Player. You can use this method to listen to events that are fired throughout the app, including:

  * `songchange` type when player changes track.
  * `onplaypause` type when player plays or pauses.
  * `onprogress` type when track progress changes.

```ts
// Register a listener that will be called when player changes track
Spicetify.Player.addEventListener("songchange", (event) => {
    // Do something
    console.log(event.data);
});
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `type` | `string` | Event type |
| `callback` | `(event?: Event) => void` | Event listener. Includes relevant information about the event (e.g. `data` for `songchange` event) |

### dispatchEvent

Dispatches an event at `Spicetify.Player`.

By default, `Spicetify.Player` always dispatch

  * `songchange` type when player changes track.
  * `onplaypause` type when player plays or pauses.
  * `onprogress` type when track progress changes.

```ts
Spicetify.Player.dispatchEvent(event);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `event` | `Event` | Event to dispatch. Includes relevant information about the event (e.g. `data` for `songchange` event) |

### back

Skip to previous track.

```ts
Spicetify.Player.back();
```

### decreaseVolume

Decrease a small amount of volume. The value is automatically determined by the client.

```ts
Spicetify.Player.decreaseVolume();
```

### formatTime

Format a time in milliseconds to a string in `mm:ss` format.

```ts
Spicetify.Player.formatTime(time);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `time` | `number` | Time in milliseconds |

#### Return

`string`

#### Example

```ts
Spicetify.Player.formatTime(1000); // "00:01"

// Get current track duration
const duration = Spicetify.Player.getDuration();
const formattedDuration = Spicetify.Player.formatTime(duration);
console.log(formattedDuration); // "03:45"
```

### getDuration

Return the duration of current track in milliseconds.

```ts
Spicetify.Player.getDuration();
```

#### Return

`number`

#### Example

```ts
// Get current track duration
const duration = Spicetify.Player.getDuration();
console.log(duration); // 225000
```

### getMute

Return the mute state of player.

```ts
Spicetify.Player.getMute();
```

#### Return

`boolean`

### getProgress

Return the progress of current track in milliseconds.

```ts
Spicetify.Player.getProgress();
```

#### Return

`number`

#### Example

```ts
// Get current track progress
const progress = Spicetify.Player.getProgress();
console.log(progress); // 10000
```

### getProgressPercent

Return the progress of current track in percentage, from 0 to 1.

```ts
Spicetify.Player.getProgressPercent();
```

#### Return

`number`

#### Example

```ts
// Get current track progress
const progress = Spicetify.Player.getProgressPercent();
console.log(progress); // 0.04
```

### getRepeat

Return the repeat mode of player. The value can be:

  * `0` for no repeat.
  * `1` for repeat all.
  * `2` for repeat one.

```ts
Spicetify.Player.getRepeat();
```

#### Return

`number`

### getShuffle

Return the shuffle state of player.

```ts
Spicetify.Player.getShuffle();
```

#### Return

`boolean`

### getHeart

Return the heart state of player.

```ts
Spicetify.Player.getHeart();
```

#### Return

`boolean`

### getVolume

Return the volume of player. The value is from 0 to 1.

```ts
Spicetify.Player.getVolume();
```

#### Return

`number`

### increaseVolume

Increase a small amount of volume. The value is automatically determined by the client.

```ts
Spicetify.Player.increaseVolume();
```

### next

Skip to next track.

```ts
Spicetify.Player.next();
```

### pause

Pause the player.

```ts
Spicetify.Player.pause();
```

### play

Resume the player.

```ts
Spicetify.Player.play();
```

### playUri

Start playback of the specified track.

```ts
Spicetify.Player.playUri(uri, context?: any, options?: any);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `uri` | `string` | Track URI string |
| `context` | `any` | Context of the track. Default is `{}` |
| `options` | `any` | Options of the track. Default is `{}` |

#### Example

```ts
// 505 - Arctic Monkeys
const trackURI = "spotify:track:0BxE4FqsDD1Ot4YuBXwAPp";

await Spicetify.Player.playUri(trackURI);
```

### removeEventListener

Unregister added event listener `type`.

```ts
Spicetify.Player.removeEventListener(type, callback);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `type` | `string` | Event type |
| `callback` | `(event?: Event) => void` | Event listener |

### seek

Seek track to position. Position can be in percentage (0 to 1) or in milliseconds.

```ts
Spicetify.Player.seek(position);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `position` | `number` | Position to seek. Can be in percentage (0 to 1) or in milliseconds |

#### Example

```ts
// Seek to 50% of track
Spicetify.Player.seek(0.5);

// Seek to 1 minute of track
Spicetify.Player.seek(60000);
```

### setHeart

Set the heart status of the currently playing track.

```ts
Spicetify.Player.setHeart(status);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `status` | `boolean` | Heart status |

### setMute

Set the mute state of player.

```ts
Spicetify.Player.setMute(state);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `state` | `boolean` | Mute state |

### setRepeat

Set the repeat mode of player. The value can be:

  * `0` for no repeat.
  * `1` for repeat all.
  * `2` for repeat one.

```ts
Spicetify.Player.setRepeat(mode);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `mode` | `number` | Repeat mode |

### setShuffle

Set the shuffle state of player.

```ts
Spicetify.Player.setShuffle(state);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `state` | `boolean` | Shuffle state |

### setVolume

Set the volume of player. The value is from 0 to 1.

```ts
Spicetify.Player.setVolume(level);
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `level` | `number` | Volume |

### toggleHeart

Toggle the heart state of player / save / unsave the current track from user's library.

```ts
Spicetify.Player.toggleHeart();
```

### toggleMute

Toggle the mute state of player.

```ts
Spicetify.Player.toggleMute();
```

### togglePlay

Toggle the play state of player.

```ts
Spicetify.Player.togglePlay();
```

### toggleRepeat

Toggle the repeat mode of player. The value switches between: No repeat, Repeat all, Repeat one.

```ts
Spicetify.Player.toggleRepeat();
```

### toggleShuffle

Toggle the shuffle state of player.

```ts
Spicetify.Player.toggleShuffle();
```


=== docs/development/api-wrapper/methods/popup-modal.md ===
---
title: PopupModal
description: Set of methods to create and control popup modals.
---

Spicetify provides a set of methods to create and control popup modals. This will display a modal on top of the client, which can be used to display information or ask for user input.

```ts
namespace PopupModal {
    interface Content {
        title: string;
        content: string | Element;
        isLarge?: boolean;
    }

    function display(content: Content): void;
    function hide(): void;
};
```

## Interface

### `Content`

`Content` is an object that contains the information needed to display the modal.

| Property | Type | Description |
| --- | --- | --- |
| `title` | `string` | Title of the modal. |
| `content` | `string` | Content of the modal. You can specify a string for simple text display or an HTML element for interactive config/setting menu. |
| `isLarge` | `boolean` &#124; `undefined` | Bigger modal. |

## Methods

### `display`

Displays a modal on top of the client.

:::note

This method will replace the current modal if there is one.

:::

| Parameter | Type | Description |
| --- | --- | --- |
| `content` | [`Content`](#content) | Information about the modal. |

```ts
Spicetify.PopupModal.display({
    title: 'Hello World',
    content: 'This is a simple text',
});
```

### `hide`

Hides the current modal.

:::note

This method will hide *any* modal currently displayed via `Spicetify.PopupModal.display`.

:::

```ts
Spicetify.PopupModal.hide();
```


=== docs/development/api-wrapper/methods/uri.md ===
---
title: URI
description: Set of API methods to parse and validate URIs.
---

Spicetify provides a set of API methods to parse and validate URIs. These are internal APIs used by Spotify in almost every component in the client.

This would be useful if you want to develop extensions that need to interact with Spotify components (e.g. open a track in the player, get playlist tracks, etc.).

:::warning

Because these APIs are internal, they are not guaranteed to be stable and may _drastically_ change in the future. Use them at your own risk.

:::

:::note

Spotify has had a major rework of this method in version `1.2.4`.

In this documentation, we will be covering the new method, used from version `1.2.4` onwards.

:::

## Introduction

Almost every component in the client has its own URI (Uniform Resource Indicator), whether it be a track, an album, a playlist, a user, etc. The URI is used to identify the resource and to interact with it.

They all follow the same format in the form of `spotify:<type>:<id>`, where `<type>` is the type of the resource and `<id>` is the `base62` unique identifier of the resource.

```
spotify:<type>:<id>
```

For example, a track URI looks like this:

```
spotify:track:6rqhFgbbKwnb9MLmUQDhG6
```

When you share a track, album, playlist, etc. on Spotify, you are actually sharing the URI of the resource.

It follows a similar format to the URI, but with a few differences:

```
https://open.spotify.com/<type>/<id>?query=parameters
```

For example, a track URL looks like this:

```
https://open.spotify.com/track/6rqhFgbbKwnb9MLmUQDhG6?si=2a3b4c5d6e7f8g9h
```

To get a URI of an item using the client interface, simply right-click on it, hover over `Share` and hold your <kbd>Ctrl</kbd> key. You will see a new option called `Copy Spotify URI` appear in place of `Copy Link`. Click on it to copy the URI to your clipboard.

## Usage

`Spicetify.URI` itself is a class that helps you create a URI object, as well as provide static methods to parse and validate URIs.

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
class URI {
  constructor(type: string, props: any);
  public type: string;
  public hasBase62Id: boolean;

  public id?: string;
  public disc?: any;
  public args?: any;
  public category?: string;
  public username?: string;
  public artist?: string;
  public album?: string;
  public query?: string;
  public country?: string;
  public global?: boolean;
  public context?: string | typeof URI | null;
  public anchor?: string;
  public play?: any;
  public toplist?: any;

  toURI(): string;
  toString(): string;
  toURLPath(opt_leadingSlash: boolean): string;
  toURL(origin?: string): string;
  clone(): URI | null;
  getPath(): string;
}
```

Almost all properties are optional, except for `type` and `hasBase62Id`. The `type` property is the type of the URI, and the `hasBase62Id` property is a boolean that indicates whether the URI has a `base62` identifier.

All other properties are dependent on the type of the URI. For example, a track URI _may_ include a `context` property, while a playlist URI _may_ include a `username` property.

The `type` property matches one of the values in the [`Spicetify.URI.Type`](/docs/development/api-wrapper/types/uri/type) enum.

### Example

```ts
const uri = new Spicetify.URI('track', {
  id: '6rqhFgbbKwnb9MLmUQDhG6',
  context: 'spotify:album:1Je1IMUlBXcx1Fz0WE7oPT',
});
```

### Dynamic methods

These methods are included in every instance of `Spicetify.URI`.

#### toURI

Creates a URI string from the URI object.

```ts
const uri = new Spicetify.URI('track', {
  id: '6rqhFgbbKwnb9MLmUQDhG6',
  context: 'spotify:album:1Je1IMUlBXcx1Fz0WE7oPT',
});

uri.toURI(); // spotify:track:6rqhFgbbKwnb9MLmUQDhG6?context=spotify:album:1Je1IMUlBXcx1Fz0WE7oPT
```

#### toString

Alias of [`toURI`](#touri)

```ts
const uri = new Spicetify.URI('track', {
  id: '6rqhFgbbKwnb9MLmUQDhG6',
  context: 'spotify:album:1Je1IMUlBXcx1Fz0WE7oPT',
});

uri.toString(); // spotify:track:6rqhFgbbKwnb9MLmUQDhG6?context=spotify:album:1Je1IMUlBXcx1Fz0WE7oPT
```

#### toURLPath

Creates a URL path from the URI object.

| Parameter        | Type                         | Description                                     |
| ---------------- | ---------------------------- | ----------------------------------------------- |
| opt_leadingSlash | `boolean` &#124; `undefined` | Whether to prepend a leading slash to the path. |

```ts
const uri = new Spicetify.URI('track', {
  id: '6rqhFgbbKwnb9MLmUQDhG6',
  context: 'spotify:album:1Je1IMUlBXcx1Fz0WE7oPT',
});

uri.toURLPath(); // /track/6rqhFgbbKwnb9MLmUQDhG6?context=spotify:album:1Je1IMUlBXcx1Fz0WE7oPT

uri.toURLPath(true); // /track/6rqhFgbbKwnb9MLmUQDhG6?context=spotify:album:1Je1IMUlBXcx1Fz0WE7oPT
```

#### toURL

Creates a URL from the URI object.

| Parameter | Type                        | Description                                                            |
| --------- | --------------------------- | ---------------------------------------------------------------------- |
| origin    | `string` &#124; `undefined` | The origin to use for the URL. Defaults to `https://open.spotify.com`. |

```ts
const uri = new Spicetify.URI('track', {
  id: '6rqhFgbbKwnb9MLmUQDhG6',
  context: 'spotify:album:1Je1IMUlBXcx1Fz0WE7oPT',
});

uri.toURL(); // https://open.spotify.com/track/6rqhFgbbKwnb9MLmUQDhG6?context=spotify:album:1Je1IMUlBXcx1Fz0WE7oPT

uri.toURL('https://example.com/'); // https://example.com/track/6rqhFgbbKwnb9MLmUQDhG6?context=spotify:album:1Je1IMUlBXcx1Fz0WE7oPT
```

#### clone

Clones the URI object.

```ts
const uri = new Spicetify.URI('track', {
  id: '6rqhFgbbKwnb9MLmUQDhG6',
  context: 'spotify:album:1Je1IMUlBXcx1Fz0WE7oPT',
});

const clone = uri.clone();

clone.toURI(); // spotify:track:6rqhFgbbKwnb9MLmUQDhG6?context=spotify:album:1Je1IMUlBXcx1Fz0WE7oPT
```

#### getPath

Gets the path of the URI object by removing all hash and query parameters.

```ts
const uri = new Spicetify.URI('track', {
  id: '6rqhFgbbKwnb9MLmUQDhG6',
  context: 'spotify:album:1Je1IMUlBXcx1Fz0WE7oPT',
});

uri.getPath(); // spotify:track:6rqhFgbbKwnb9MLmUQDhG6
```

### Static methods

These methods are included in the `Spicetify.URI` class.

#### Type

An enum of all URI types.

```ts
const { Type } = Spicetify.URI;

Type.ALBUM; // album
```

For a list of all URI types, see the [`Spicetify.URI.Type`](/docs/development/api-wrapper/types/uri/type) enum and explore in DevTools.

##### Usage

You can use this enum to check and validate the type of a URI.

```ts
const uri = new Spicetify.URI('track', {
  id: '6rqhFgbbKwnb9MLmUQDhG6',
  context: 'spotify:album:1Je1IMUlBXcx1Fz0WE7oPT',
});

uri.type === Spicetify.URI.Type.TRACK; // true
```

#### from

Parses a given argument into a [`URI`](#usage) instance.

Unlike [`URI.fromString`](#fromstring), this function could receive any kind of value.

If the value is already a URI instance, it is simply returned. Otherwise the value will be stringified before parsing.

This function also does not throw an error like [`URI.fromString`](#fromstring), but
instead simply returns null if it can't parse the value.

```ts
class URI {
  static from(value: any): URI | null;
}
```

| Parameter | Type  | Description         |
| --------- | ----- | ------------------- |
| value     | `any` | The value to parse. |

```ts
Spicetify.URI.from('spotify:track:6rqhFgbbKwnb9MLmUQDhG6'); // URI instance

Spicetify.URI.from('https://open.spotify.com/track/6rqhFgbbKwnb9MLmUQDhG6'); // URI instance

Spicetify.URI.from(new Spicetify.URI('track', { id: '6rqhFgbbKwnb9MLmUQDhG6' })); // URI instance

Spicetify.URI.from(Spicetify.URI.from('spotify:track:6rqhFgbbKwnb9MLmUQDhG6')); // URI instance

Spicetify.URI.from({ id: '6rqhFgbbKwnb9MLmUQDhG6' }); // null
```

#### fromString

Parses a given string into a [`URI`](#usage) instance.

:::caution

This function will throw a `TypeError` if the argument passed is not a string. Use [`URI.from`](#from) if you want to parse any kind of value.

:::

```ts
class URI {
  static fromString(uri: string): URI;
}
```

| Parameter | Type     | Description       |
| --------- | -------- | ----------------- |
| uri       | `string` | The URI to parse. |

```ts
Spicetify.URI.fromString('spotify:track:6rqhFgbbKwnb9MLmUQDhG6'); // URI instance

Spicetify.URI.fromString('https://open.spotify.com/track/6rqhFgbbKwnb9MLmUQDhG6'); // URI instance

Spicetify.URI.fromString('spotify:track:6rqhFgbbKwnb9MLmUQDhG6?context=spotify:album:1Je1IMUlBXcx1Fz0WE7oPT'); // URI instance

Spicetify.URI.fromString(new Spicetify.URI('track', { id: '6rqhFgbbKwnb9MLmUQDhG6' })); // TypeError: Argument `uri` must be a string.
```

#### Validation functions

Each URI type has a validation function that can be used to check if a given string is a valid URI of that type.

For a list of all validation functions, see [`Validation functions`](/docs/development/api-wrapper/types/uri/validation-functions).

:::caution

Almost all playlists use the `playlist-v2` type, so use `Spicetify.URI.isPlaylistV2URI` instead.

Similarly, you can use `Spicetify.URI.isPlaylistV1OrV2` to check if a URI is a playlist of any version.

:::

```ts
const { isAlbumURI } = Spicetify.URI;

isAlbumURI('spotify:album:1Je1IMUlBXcx1Fz0WE7oPT'); // true
```


=== docs/development/api-wrapper/modules.md ===
---
title: Modules
description: 🧩 Modules exposed via Spicetify object.
---

Spicetify exposes some modules via `Spicetify` object.

You can access them by typing `Spicetify.<module name>` in the DevTools console, inside your extension, or `window.top.Spicetify.<module name>` if you're developing an app inside an `iframe`.

Utilizing these modules can help you create more powerful extensions without having to include the whole module in your extension.

```js
Spicetify.React;
```

For usage of these modules, please refer to their official documentation.

### React

[React](https://reactjs.org/) is a JavaScript library for building user interfaces. It is used by Spotify to build their UI.

:::note

Spotify versions *below* 1.2.26 use version **17.0.2**, *after* - **18.2.0**.

:::

```js
Spicetify.React;
```

### ReactDOM

[ReactDOM](https://reactjs.org/docs/react-dom.html) is a package that provides DOM-specific methods that can be used at the top level of your app and as an escape hatch to get outside of the React model if you need to. It is used by Spotify to render React components to the DOM.

```js
Spicetify.ReactDOM;
```

### Tippy.js

[Tippy.js](https://atomiks.github.io/tippyjs/) is a highly customizable tooltip and popover library powered by Popper.

```js
Spicetify.Tippy;
```

### Mousetrap

[Mousetrap](https://craig.is/killing/mice) is a simple library for handling keyboard shortcuts in JavaScript.

```js
Spicetify.Mousetrap;
```

### React Flip Toolkit

[React Flip Toolkit](https://github.com/aholachek/react-flip-toolkit) is a collection of easy-to-use animation effects and utilities that can be used to enhance your React project.

```js
Spicetify.ReactFlipToolkit;
```

### React Query (v3)

[React Query](https://react-query.tanstack.com/) is a library for managing, caching, syncing, and refetching server state in React.

:::note

Spotify uses React Query v3, instead of the current latest version (v4). As such, the API may be different from the official documentation.

:::

```js
Spicetify.ReactQuery;
```

### classnames

[classnames](https://github.com/JedWatson/classnames) is a simple JavaScript utility for conditionally joining class names together.

```js
Spicetify.classnames;
```

### Snackbar

[Notistack](https://github.com/iamhosseindhv/notistack) is a JavaScript library for creating highly customizable notification snackbars (toasts) that can be stacked on top of each other.

:::note

Be aware that only `SnackbarProvider` and `useSnackbar` work as described in the official Notistack documentation.

:::

```js
Spicetify.Snackbar;
```


=== docs/development/api-wrapper/properties/config.md ===
---
title: Config
description: 🛠️ Accessing a copy of Spicetify's `config-xpui.ini` file inside your extension.
---

To make it easier for you to validate and debug your extensions, Spicetify provides a filtered copy of the user's `config-xpui.ini` in the `Spicetify` object.

```ts
interface Config {
  version: string;
  current_theme: string;
  color_scheme: string;
  extensions: string[];
  custom_apps: string[];
}
```

| Property | Type | Description |
| --- | --- | --- |
| `version` | `string` | Spicetify version. |
| `current_theme` | `string` | Current theme name. |
| `color_scheme` | `string` | Current color scheme name. |
| `extensions` | `string[]` | List of enabled extensions. |
| `custom_apps` | `string[]` | List of enabled custom apps. |

## Usage

You can validate if the user currently has a custom app or a theme enabled by checking if the app or theme's name is included in the `custom_apps` or `current_theme` property of the `Config` object.

```ts
const { Config } = Spicetify;

if (Config.custom_apps.includes("lyrics-plus")) {
    // Do something
}
```

This can ensure that your extension doesn't break if the user doesn't have the required app or theme installed.


=== docs/development/api-wrapper/properties/queue.md ===
---
title: Queue
description: An object containing information about the current queue.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

The `Queue` object contains a list of queuing tracks, history of played tracks, and current track metadata.

```ts
Spicetify.Queue
```

## Return

```ts
const Queue: {
    nextTracks: ProvidedTrack[];
    prevTracks: ProvidedTrack[];
    queueRevision: string;
    track: ProvidedTrack;
};
```

| Property | Type | Description |
| --- | --- | --- |
| `nextTracks` | [`ProvidedTrack[]`](/docs/development/api-wrapper/types/provided-track) | List of next tracks. |
| `prevTracks` | [`ProvidedTrack[]`](/docs/development/api-wrapper/types/provided-track) | List of previous tracks. |
| `queueRevision` | `string` | Queue revision ID used internally by Spotify. |
| `track` | [`ProvidedTrack`](/docs/development/api-wrapper/types/provided-track) | Current track. |

## Usage

If you plan on developing extensions that need to access the current queue, you can use the `Spicetify.Queue` object.

```ts
const queue = Spicetify.Queue;
const currentTrack = queue.track;
```


=== docs/development/api-wrapper/properties/react-components.md ===
---
title: ReactComponent
description: Set of stock React components used by Spotify.
---

Spicetify provides a set of stock React components used by Spotify. You can use these components to create your own custom UI.

:::note

It is recommended that you be familiar with [`React`](https://react.dev/) and [`spicetify-creator`](/docs/development/spicetify-creator/the-basics) before using these components.

:::

```ts
namespace ReactComponent {
    const ContextMenu: any;
    const RightClickMenu: any;
    const Menu: any;
    const MenuItem: any;
    const AlbumMenu: any;
    const PodcastShowMenu: any;
    const ArtistMenu: any;
    const PlaylistMenu: any;
    const TooltipWrapper: any;
    const IconComponent: any;
    const TextComponent: any;
    const ConfirmDialog: any;
    const PanelSkeleton: any;
    const PanelContent: any;
    const PanelHeader: any;
    const Toggle: any;
    const Slider: any
};
```

## Components

:::info

These components may be wrappers for other components such as [`Tippy`](https://atomiks.github.io/tippyjs/) or [`styled-components`](https://styled-components.com/). They may accept additional props that are not listed here.

As such, type definitions are not forced for these components but they act as a guideline for what you can use and what Spotify uses.

Refer to the underlying library's documentation for more information.

:::

### `ContextMenu`

Generic context menu provider. It is used by Spotify on a variety of elements, such as right-click menu, dropdown menu, etc.

#### Props

See [`ContextMenuProps`](/docs/development/api-wrapper/types/react-component/context-menu-props).

#### Example

```tsx
// See Menu section for more details
const menuWrapper = React.memo((props: MenuProps) =>
    <Spicetify.ReactComponent.Menu {...props}>
        <Spicetify.ReactComponent.MenuItem
            label="Hello World"
            onClick={() => Spicetify.showNotification('Hello World')}
        />
    </Spicetify.ReactComponent.Menu>
});

const contextMenu = React.memo((props: ContextMenuProps) => {
    return (
        <Spicetify.ReactComponent.ContextMenu {...props}
            trigger="click"
            menu={<menuWrapper {...props} />}
        >
            <div>Click me</div>
        </Spicetify.ReactComponent.ContextMenu>
    );
});
```

### `RightClickMenu`

Wrapper of [`ContextMenu`](#contextmenu) with predefined props: `action = 'toggle'` and `trigger = 'right-click'`.

#### Props

See [`ContextMenuProps`](/docs/development/api-wrapper/types/react-component/context-menu-props).

#### Example

```tsx
const menuWrapper = React.memo((props: MenuProps) =>
    <Spicetify.ReactComponent.Menu {...props}>
        <Spicetify.ReactComponent.MenuItem
            label="Hello World"
            onClick={() => Spicetify.showNotification('Hello World')}
        />
    </Spicetify.ReactComponent.Menu>
});
// Same as ContextMenu example, but appears on right-click
const contextMenu = React.memo((props: ContextMenuProps) => {
    return (
        <Spicetify.ReactComponent.RightClickMenu {...props}
            menu={<menuWrapper {...props} />}
        >
            <div>Right-click me</div>
        </Spicetify.ReactComponent.RightClickMenu>
    );
});
```

### `Menu`

Outer layer containing [`MenuItem`](#menuitem)s.

#### Props

See [`MenuProps`](/docs/development/api-wrapper/types/react-component/menu-props).

#### Example

```tsx
const menuWrapper = React.memo((props: MenuProps) =>
    <Spicetify.ReactComponent.Menu {...props} onClose={() => Spicetify.showNotification('Menu closed')}>
        <Spicetify.ReactComponent.MenuItem
            label="Hello World"
            onClick={() => Spicetify.showNotification('Hello World')}
        />
    </Spicetify.ReactComponent.Menu>
});
```

### `MenuItem`

Component to construct menu item. Used as [`Menu`](#menu) children.

#### Props

See [`MenuItemProps`](/docs/development/api-wrapper/types/react-component/menu-item-props).

#### Example

```tsx
const icon = React.memo((props: IconComponentProps) =>
    <Spicetify.ReactComponent.IconComponent {...props}
        semanticColor="textBase"
        dangerouslySetInnerHTML={{ __html: Spicetify.SVGIcons["play"] }}
        iconSize={16}
    />
);

const menuItem = React.memo((props: MenuItemProps) =>
    <Spicetify.ReactComponent.MenuItem {...props}
        onClick={() => Spicetify.showNotification('Hello World')}
        disabled={false}
        divider="after"
        {/* It is recommended that you use both `icon` and `trailingIcon` for compatibility between older versions */}
        icon={<icon />}
        trailingIcon={<icon />}
    >
        Hello World
    </Spicetify.ReactComponent.MenuItem>
);
```

### `AlbumMenu`, `PodcastShowMenu`, `ArtistMenu`, `PlaylistMenu`

Tailored [`Menu`](#menu) for specific type of object.

#### Props

Accepts `uri` and `onRemoveCallback` props along with [`MenuProps`](/docs/development/api-wrapper/types/react-component/menu-props).

```ts
interface AlbumMenuProps extends MenuProps {
    uri: string;
    onRemoveCallback?: (uri: string) => void;
};
```

#### Example

```tsx
const currentAlbumURI = Spicetify.Player.data.item.metadata.album_uri;

const albumMenu = React.memo((props: AlbumMenuProps) =>
    <Spicetify.ReactComponent.AlbumMenu {...props}
        onClose={() => Spicetify.showNotification('Menu closed')}
        uri={currentAlbumURI}
    />
);
```

### `TooltipWrapper`

Component to display tooltip when hovering over element. Useful for accessibility.

:::info

This component is a wrapper for [`Tippy`](https://atomiks.github.io/tippyjs/). It may accept additional props that are not listed here.

:::

#### Props

See [`TooltipProps`](/docs/development/api-wrapper/types/react-component/tooltip-props).

#### Example

```tsx
const elementHasTooltip = React.memo((props: TooltipProps) =>
    <Spicetify.ReactComponent.TooltipWrapper {...props}
        label="Hello World"
        placement="bottom"
    >
        <div>Hover me</div>
    </Spicetify.ReactComponent.TooltipWrapper>
);
```

### `IconComponent`

Component to render Spotify-style icon. It is used by Spotify on a variety of elements, such as buttons, icons, etc.

:::info

This component is a wrapper for [`styled-components`](https://styled-components.com/). It may accept additional props that are not listed here.

:::

#### Props

See [`IconComponentProps`](/docs/development/api-wrapper/types/react-component/icon-component-props).

#### Example

```tsx
const icon = React.memo((props: IconComponentProps) =>
    <Spicetify.ReactComponent.IconComponent {...props}
        semanticColor="textBase"
        dangerouslySetInnerHTML={{ __html: Spicetify.SVGIcons["play"] }}
        iconSize={16}
    />
);
```

### `TextComponent`

Component to render text. It is used by Spotify on a variety of elements, such as buttons, text, etc.

:::info

This component is a wrapper for [`styled-components`](https://styled-components.com/). It may accept additional props that are not listed here.

:::

#### Props

See [`TextComponentProps`](/docs/development/api-wrapper/types/react-component/text-component-props).

#### Example

```tsx
const text = React.memo((props: TextComponentProps) =>
    <Spicetify.ReactComponent.TextComponent {...props}
        semanticColor="textBase"
        variant="viola"
        weight="book"
    >
        Hello World
    </Spicetify.ReactComponent.TextComponent>
);
```

### `ConfirmDialog`

Component to display Spotify-style confirmation dialog. Used by Spotify on playlist, track removal, etc.

:::info

For each of the `onConfirm`, `onCancel`, and `onOutsideClick` props, the dialog will not close automatically. You must manually handle the state of the dialog.

:::

#### Props

See [`ConfirmDialogProps`](/docs/development/api-wrapper/types/react-component/confirm-dialog-props).

#### Example

```tsx
const ConfirmButton = () => {
    // Modal open state must be handled manually
    const [showModal, setShowModal] = React.useState(false);

    return (
        <Spicetify.ReactComponent.ConfirmDialog
            isOpen={showModal}
            onConfirm={() => {
                setShowModal(false);
                Spicetify.showNotification('Confirmed');
            }}
            onCancel={() => {
                setShowModal(false);
                Spicetify.showNotification('Cancelled');
            }}
            onOutsideClick={() => {
                setShowModal(false);
                Spicetify.showNotification('Clicked outside');
            }}
            titleText="Confirm Modal"
            descriptionText="Are you sure you want to confirm?"
            confirmText="Confirm"
            cancelText="Cancel"
        />
        <button onClick={() => setShowModal(true)}>Click me</button>
    );
}
```

### `Toggle`

Component to display Spotify-style toggle. Used by Spotify on the settings page.

#### Props

See [`ToggleProps`](/docs/development/api-wrapper/types/react-component/toggle-props).

```tsx
const Toggle = () => {
    const [enabled, setEnabled] = React.useState(false);

    return (
        <Spicetify.ReactComponent.Toggle
            value={enabled}
            onSelected={setEnabled}
            id="my-toggle-id"
            class="my-toggle-class"
        ></Spicetify.ReactComponent.Toggle>
    );
}
```

### `Slider`

Component to render sliders. It is used by Spotify for the volume/playing bars and on the settings page.

#### Props

See [`SliderProps`](/docs/development/api-wrapper/types/react-component/slider-props).

#### Example

```tsx
const Slider = () => {
    const [value, setValue] = useState(0);

    return (
        <Spicetify.ReactComponent.Slider
            max={100}
            step={1}
            value={value}
            onDragStart={() => {}}
            onDragMove={setValue}
            onDragEnd={(value) => {console.log(`final value is ${value}`)}}
        ></Spicetify.ReactComponent.Slider>
    );
}
```

### `PanelSkeleton`, `PanelContent`, `PanelHeader`

Components to render Spotify-style panel. Used by Spotify on their right sidebar panels (e.g. BuddyFeed, Now Playing, etc).

Refer to [`Panel.Components`](/docs/development/api-wrapper/methods/panel#components) for more details.

=== docs/development/api-wrapper/properties/react-hook.md ===
---
title: ReactHook
description: Set of React hooks used by the Spotify client.
---

Spicetify provides a set of React hooks used by the Spotify client. You can use these hooks to create a React component interactive with the client.

:::note

It is recommended that you be familiar with [`React`](https://react.dev/) before using these hooks.

:::

```ts
namespace ReactHook {
    function DragHandler(
        uris?: string[],
        label?: string,
        contextUri?: string,
        sectionIndex?: number,
        dropOriginUri?: string
    ): (event: React.DragEvent, uris?: string[], label?: string, contextUri?: string, sectionIndex?: number) => void;
    function usePanelState(id: number): { toggle: () => void, isActive: boolean };
    function useExtractedColor(uri: string, fallbackColor?: string, variant?: "colorRaw" | "colorLight" | "colorDark"): string;
}
```

## Hooks

### `DragHandler`

React Hook to create interactive drag-and-drop element.

Used to create a draggable element that can be dropped into Spotify's components (e.g. Playlist, Folder, Sidebar, Queue)

```ts
function DragHandler(
    uris?: string[],
    label?: string,
    contextUri?: string,
    sectionIndex?: number,
    dropOriginUri?: string
): (event: React.DragEvent, uris?: string[], label?: string, contextUri?: string, sectionIndex?: number) => void;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| uris | `string[]` &#124; `undefined` | List of URIs to be dragged. |
| label | `string` &#124; `undefined` | Label to be displayed when dragging. |
| contextUri | `string` &#124; `undefined` | Context URI of the element from which the drag originated (e.g. Playlist URI). |
| sectionIndex | `number` &#124; `undefined` | Index of the section in which the drag originated. |
| dropOriginUri | `string` &#124; `undefined` | URI of the desired drop target. Leave empty to allow drop anywhere. |

#### Returns

Function to handle drag event. Should be passed to `onDragStart` prop of the element. All parameters passed onto the hook will be passed onto the handler unless declared otherwise.

#### Example

```tsx
const DraggableComponent = () => {
    // Do I Wanna Know? by Arctic Monkeys
    const uri = "spotify:track:5FVd6KXrgO9B3JPmC8OPst";
    const label = "Do I Wanna Know? - Arctic Monkeys";

    const handleDragStart = Spicetify.ReactHook.DragHandler([uri], label);

    return (
        <div draggable onDragStart={handleDragStart}>
            {label}
        </div>
    );
}
```

### `usePanelState`

React Hook to use panel state.

```ts
function usePanelState(id: number): { toggle: () => void, isActive: boolean };
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| id | `number` | ID of the panel to use. |

#### Returns

Object with methods of the panel.

| Property | Type | Description |
| :--- | :--- | :--- |
| toggle | `() => void` | Toggle the panel. |
| isActive | `boolean` | Whether the panel is active. |

#### Example

```tsx
const PanelComponent = () => {
    // The ID can be either Spotify's default panel IDs or your custom panel ID registered via `Spicetify.Panel.registerPanel`
    const { toggle, isActive } = Spicetify.ReactHook.usePanelState(5);

    return (
        <div>
            <button onClick={toggle}>
                {isActive ? "Close" : "Open"} Panel
            </button>
        </div>
    );
}
```

### `useExtractedColor`

React Hook to use extracted color from GraphQL.

```ts
function useExtractedColor(uri: string, fallbackColor?: string, variant?: "colorRaw" | "colorLight" | "colorDark"): string;
```

:::note

This is a wrapper of ReactQuery's `useQuery` hook. The component using this hook must be wrapped in a `QueryClientProvider` component.

Look into the example below for more information.

:::

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| uri | `string` | URI of the Spotify image to extract color from. |
| fallbackColor | `string` &#124; `undefined` | Fallback color to use if the image is not available. Defaults to `#535353`. |
| variant | `"colorRaw"` &#124; `"colorLight"` &#124; `"colorDark"` &#124; `undefined` | Variant of the color to use. Defaults to `colorRaw`. |

#### Returns

Extracted color hex code.

#### Example

```tsx
import { useEffect, useState } from "react";

const { QueryClient, QueryClientProvider } = Spicetify.ReactQuery;
const { useExtractedColor } = Spicetify.ReactHook;

const queryClient = new QueryClient();

const Component = () => {
    const [imageUri, setImageUri] = useState(Spicetify.Player.data?.item?.metadata?.image_xlarge_url ?? "");
    const color = useExtractedColor(imageUri);

    useEffect(() => {
        // Listen to track change
        const listener = () => {
            setImageUri(Spicetify.Player.data?.item?.metadata?.image_xlarge_url ?? "");
        };
        Spicetify.Player.addEventListener("songchange", listener);

        return () => Spicetify.Player.removeEventListener("songchange", listener);
    }, []);

    return (
        <div style={{ backgroundColor: color }}>
            Hello World
        </div>
    );
}

const App = () => {
    return (
        <QueryClientProvider client={queryClient}>
            <Component />
        </QueryClientProvider>
    );
}
```


=== docs/development/api-wrapper/properties/svgicons.md ===
---
title: SVGIcons
description: A set of SVG icons used throughout the Spotify client.
---

Spicetify has a predefined set of SVG icons that are used by Spotify throughout the client. These are strings of SVG `innerHTML` that are used to create `<svg>` elements.

```ts
const SVGIcons = Record<SVGIcon, string>;
```

| Property | Type | Description |
| --- | --- | --- |
| `key` | [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) | SVG icon name. |

## Usage

You can use these icons to create custom menu items or other custom components.

In vanilla JavaScript, you can create an `<svg>` element and set its `innerHTML` to the SVG icon string.

```ts
const icon = document.createElement("svg");
icon.innerHTML = Spicetify.SVGIcons["play"];
```

In React, you can use the `dangerouslySetInnerHTML` prop to set the SVG icon string as the inner HTML of the `<svg>` element.

```tsx
const icon = <svg dangerouslySetInnerHTML={{ __html: Spicetify.SVGIcons["play"] }} />;
```

In Spicetify's own methods, you can simply pass an [`SVGIcon`](/docs/development/api-wrapper/types/svgicon) to the `icon` parameter.

```ts
new Spicetify.ContextMenu.Item(
  name: "My Custom Item".
  onClick: () => Spicetify.showNotification("Hello World!"),
  shouldAdd: () => true,
  icon: "play",
  disabled: false,
)
```


=== docs/development/api-wrapper/properties/tippy-props.md ===
---
title: TippyProps
description: Predefined props for Tippy.js tooltips.
---

Spicetify provides a set of predefined props for Tippy.js tooltips. This is aimed to create tooltips that mimic the style of Spotify's tooltips.

This is utilized for [`Topbar`](/docs/development/api-wrapper/classes/topbar) and [`Playbar`](/docs/development/api-wrapper/classes/playbar) tooltips.

```ts
Spicetify.TippyProps = {
    delay: [200, 0],
    animation: true,
    render(instance) {
        const popper = document.createElement('div');
        const box = document.createElement('div');

        popper.id = "context-menu";
        popper.appendChild(box);

        box.className = "main-contextMenu-tippy"
        box.textContent = instance.props.content;

        function onUpdate(prevProps, nextProps) {
            if (prevProps.content !== nextProps.content) {
            if (nextProps.allowHTML) box.innerHTML = nextProps.content;
            else box.textContent = nextProps.content;
            }
        }

        return { popper, onUpdate }
    },
    onShow(instance) {
        instance.popper.firstChild.classList.add("main-contextMenu-tippyEnter");
    },
    onMount(instance) {
        requestAnimationFrame(() => {
            instance.popper.firstChild.classList.remove("main-contextMenu-tippyEnter");
            instance.popper.firstChild.classList.add("main-contextMenu-tippyEnterActive");
        });
    },
    onHide(instance) {
        requestAnimationFrame(() => {
            instance.popper.firstChild.classList.remove("main-contextMenu-tippyEnterActive");
            instance.unmount();
        });
    },
},
```

#### Usage

If you want to use this set of props for your own Tippy.js tooltips, you can simply spread the `Spicetify.TippyProps` object into your Tippy.js instance.

```ts
const element = document.createElement("div");

const tooltip = tippy(element, {
    ...Spicetify.TippyProps,
    content: "Tooltip content",
    // For example, if you want to override the delay
    delay: [100, 0],
});
```


=== docs/development/api-wrapper/types/context-menu/onclick-callback.md ===
---
title: OnClickCallback
description: Type definition for callback function when menu item is clicked.
---

```ts
type OnClickCallback = (
    uris: string[],
    uids?: string[],
    contextUri?: string
) => void;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| uris | `string[]` | List of URIs of the selected items. |
| uids | `string[]` &#124; `undefined` | List of UIDs of the selected items. **Note:** Not all context menu items have UIDs. |
| contextUri | `string` &#124; `undefined` | URI of the context menu where the item was called from. This could be a playlist, album, artist, or a track. |


=== docs/development/api-wrapper/types/context-menu/should-add-callback.md ===
---
title: ShouldAddCallback
description: Type definition for callback function to determine if menu item should be added.
---

```ts
type ShouldAddCallback = (
    uris: string[],
    uids?: string[],
    contextUri?: string
) => boolean;
```

#### Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| uris | `string[]` | List of URIs of the selected items. |
| uids | `string[]` &#124; `undefined` | List of UIDs of the selected items. **Note:** Not all context menu items have UIDs. |
| contextUri | `string` &#124; `undefined` | URI of the context menu where the item was called from. This could be a playlist, album, artist, or a track. |

#### Return value

`boolean` - Whether the menu item should be added.


=== docs/development/api-wrapper/types/context-option.md ===
---
title: ContextOption
description: ContextOption type definition.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type ContextOption = {
    contextURI?: string;
    index?: number;
    trackUri?: string;
    page?: number;
    trackUid?: string;
    sortedBy?: string;
    filteredBy?: string;
    shuffleContext?: boolean;
    repeatContext?: boolean;
    repeatTrack?: boolean;
    offset?: number;
    next_page_url?: string;
    restrictions?: Record<string, string[]>;
    referrer?: string;
};
```

| Property | Type | Description |
| --- | --- | --- |
| `contextURI` | `string` &#124; `undefined` | Context URI. |
| `index` | `number` &#124; `undefined` | Track index. |
| `trackUri` | `string` &#124; `undefined` | Track URI. |
| `page` | `number` &#124; `undefined` | Page number. |
| `trackUid` | `string` &#124; `undefined` | Track UID. |
| `sortedBy` | `string` &#124; `undefined` | Sorted by timestamp. |
| `filteredBy` | `string` &#124; `undefined` | Filtered by timestamp. |
| `shuffleContext` | `boolean` &#124; `undefined` | Shuffle context URI. |
| `repeatContext` | `boolean` &#124; `undefined` | Repeat context URI. |
| `repeatTrack` | `boolean` &#124; `undefined` | Repeat track URI. |
| `offset` | `number` &#124; `undefined` | Offset. |
| `next_page_url` | `string` &#124; `undefined` | Next page URL. |
| `restrictions` | `Record<string, string[]>` &#124; `undefined` | Restrictions. |
| `referrer` | `string` &#124; `undefined` | Referrer. |


=== docs/development/api-wrapper/types/context-track.md ===
---
title: ContextTrack
description: ContextTrack type definition.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type ContextTrack = {
    uri: string;
    uid?: string | null;
    metadata?: Metadata;
}
```

| Property | Type | Description |
| --- | --- | --- |
| `uri` | `string` | Track URI. |
| `uid` | `string` &#124; `undefined` &#124; `null` | Track UID. |
| `metadata` | [`Metadata`](/docs/development/api-wrapper/types/metadata) &#124; `undefined` | Track metadata. |


=== docs/development/api-wrapper/types/cosmos-async/body.md ===
---
title: Body
description: CosmosAsync Body type definition.
---

Parsed JSON response body.

```ts
type Body = Record<string, any>;
```


=== docs/development/api-wrapper/types/cosmos-async/error.md ===
---
title: Error
description: CosmosAsync Error type definition.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
interface Error {
    code: number;
    error: string;
    message: string;
    stack?: string;
};
```

| Property | Type | Description |
| --- | --- | --- |
| `code` | `number` | HTML error code. |
| `error` | `string` | HTML error name. |
| `message` | `string` | Error message. |
| `stack` | `string` &#124; `undefined` | Error stack. |


=== docs/development/api-wrapper/types/cosmos-async/headers.md ===
---
title: Headers
description: CosmosAsync Headers type definition.
---

Equivalent to `XMLHttpRequest`'s `headers` property.

```ts
type Headers = Record<string, string>;
```


=== docs/development/api-wrapper/types/cosmos-async/method.md ===
---
title: Method
description: CosmosAsync Method type definition.
---

Equivalent to `XMLHttpRequest`'s `method` property.

```ts
type Method = "DELETE" | "GET" | "HEAD" | "PATCH" | "POST" | "PUT" | "SUB";
```


=== docs/development/api-wrapper/types/cosmos-async/response.md ===
---
title: Response
description: CosmosAsync Response type definition.
---

Represents a response from a CosmosAsync request.

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
interface Response {
    body: any;
    headers: Headers;
    status: number;
    uri?: string;
}
```

| Property | Type | Description |
| --- | --- | --- |
| `body` | [`Body`](./body.md) | Parsed JSON response body. |
| `headers` | [`Headers`](./headers.md) | Response headers. |
| `status` | `number` | HTTP status code. |
| `uri` | `string` &#124; `undefined` | Request URI. |


=== docs/development/api-wrapper/types/graphql/query.md ===
---
title: Query
description: List of GraphQL definitions used by Spotify.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type Query = "decorateItemsForEnhance" |
    "imageURLAndSize" |
    "imageSources" |
    "audioItems" |
    "creator" |
    "extractedColors" |
    "extractedColorsAndImageSources" |
    "fetchExtractedColorAndImageForAlbumEntity" |
    "fetchExtractedColorAndImageForArtistEntity" |
    "fetchExtractedColorAndImageForEpisodeEntity" |
    "fetchExtractedColorAndImageForPlaylistEntity" |
    "fetchExtractedColorAndImageForPodcastEntity" |
    "fetchExtractedColorAndImageForTrackEntity" |
    "fetchExtractedColorForAlbumEntity" |
    "fetchExtractedColorForArtistEntity" |
    "fetchExtractedColorForEpisodeEntity" |
    "fetchExtractedColorForPlaylistEntity" |
    "fetchExtractedColorForPodcastEntity" |
    "fetchExtractedColorForTrackEntity" |
    "getAlbumNameAndTracks" |
    "getEpisodeName" |
    "getTrackName" |
    "queryAlbumTrackUris" |
    "queryTrackArtists" |
    "decorateContextEpisodesOrChapters" |
    "decorateContextTracks" |
    "fetchTracksForRadioStation" |
    "decoratePlaylists" |
    "playlistUser" |
    "FetchPlaylistMetadata" |
    "playlistContentsItemTrackArtist" |
    "playlistContentsItemTrackAlbum" |
    "playlistContentsItemTrack" |
    "playlistContentsItemLocalTrack" |
    "playlistContentsItemEpisodeShow" |
    "playlistContentsItemEpisode" |
    "playlistContentsItemResponse" |
    "playlistContentsItem" |
    "FetchPlaylistContents" |
    "episodeTrailerUri" |
    "podcastEpisode" |
    "podcastMetadataV2" |
    "minimalAudiobook" |
    "audiobookChapter" |
    "audiobookMetadataV2" |
    "fetchExtractedColors" |
    "queryFullscreenMode" |
    "queryNpvEpisode" |
    "queryNpvArtist" |
    "albumTrack" |
    "getAlbum" |
    "queryAlbumTracks" |
    "queryArtistOverview" |
    "queryArtistAppearsOn" |
    "discographyAlbum" |
    "albumMetadataReleases" |
    "albumMetadata" |
    "queryArtistDiscographyAlbums" |
    "queryArtistDiscographySingles" |
    "queryArtistDiscographyCompilations" |
    "queryArtistDiscographyAll" |
    "queryArtistDiscographyOverview" |
    "artistPlaylist" |
    "queryArtistPlaylists" |
    "queryArtistDiscoveredOn" |
    "queryArtistFeaturing" |
    "queryArtistRelated" |
    "queryArtistMinimal" |
    "searchModalResults" |
    "queryWhatsNewFeed" |
    "whatsNewFeedNewItems" |
    "SetItemsStateInWhatsNewFeed" |
    "browseImageURLAndSize" |
    "browseImageSources" |
    "browseAlbum" |
    "browseArtist" |
    "browseEpisode" |
    "browseChapter" |
    "browsePlaylist" |
    "browsePodcast" |
    "browseAudiobook" |
    "browseTrack" |
    "browseUser" |
    "browseMerch" |
    "browseArtistConcerts" |
    "browseContent" |
    "browseSectionContainer" |
    "browseClientFeature" |
    "browseItem" |
    "browseAll" |
    "browsePage";
```

=== docs/development/api-wrapper/types/keyboard/keysdefine.md ===
---
title: KeysDefine
description: Keyboard KeyDefine type definition.
---

```ts
type KeysDefine = string | {
    key: string;
    ctrl?: boolean;
    shift?: boolean;
    alt?: boolean;
    meta?: boolean;
};
```

`KeysDefine` is a type that defines a keyboard shortcut. It can be a string or an object.

In the string format, it should be a list of keys separated by `+`. For example, `ctrl+shift+p` is a valid shortcut.

In the object format, it should be an object with the following properties:

| Property | Type | Description |
| --- | --- | --- |
| `key` | `string` | Key name. Refer to [this table](/docs/development/api-wrapper/types/keyboard/validkey) for a list of valid keys. |
| `ctrl` | `boolean` &#124; `undefined` | Whether to require `CTRL` key. |
| `shift` | `boolean` &#124; `undefined` | Whether to require `SHIFT` key. |
| `alt` | `boolean` &#124; `undefined` | Whether to require `ALT` key. |
| `meta` | `boolean` &#124; `undefined` | Whether to require the meta key. This could be the <kbd>🪟</kbd> key on Windows or the <kbd>⌘</kbd> key on Mac. |


=== docs/development/api-wrapper/types/keyboard/validkey.md ===
---
title: ValidKey
description: A list of valid keys for keyboard shortcuts.
---

:::tip

For the list of valid keys, refer to the [type definition](#type-definition).

:::

### Key/value pairs

|Key|Value|
|---|---|
|BACKSPACE|backspace|
|TAB|tab|
|ENTER|enter|
|SHIFT|shift|
|CTRL|ctrl|
|ALT|alt|
|CAPS|capslock|
|ESCAPE|esc|
|SPACE|space|
|PAGE_UP|pageup|
|PAGE_DOWN|pagedown|
|END|end|
|HOME|home|
|ARROW_LEFT|left|
|ARROW_UP|up|
|ARROW_RIGHT|right|
|ARROW_DOWN|down|
|INSERT|ins|
|DELETE|del|
|A|a|
|B|b|
|C|c|
|D|d|
|E|e|
|F|f|
|G|g|
|H|h|
|I|i|
|J|j|
|K|k|
|L|l|
|M|m|
|N|n|
|O|o|
|P|p|
|Q|q|
|R|r|
|S|s|
|T|t|
|U|u|
|V|v|
|W|w|
|X|x|
|Y|y|
|Z|z|
|WINDOW_LEFT|meta|
|WINDOW_RIGHT|meta|
|SELECT|meta|
|NUMPAD_0|0|
|NUMPAD_1|1|
|NUMPAD_2|2|
|NUMPAD_3|3|
|NUMPAD_4|4|
|NUMPAD_5|5|
|NUMPAD_6|6|
|NUMPAD_7|7|
|NUMPAD_8|8|
|NUMPAD_9|9|
|MULTIPLY|*|
|ADD|+|
|SUBTRACT|-|
|DECIMAL_POINT|.|
|DIVIDE|/|
|F1|f1|
|F2|f2|
|F3|f3|
|F4|f4|
|F5|f5|
|F6|f6|
|F7|f7|
|F8|f8|
|F9|f9|
|F10|f10|
|F11|f11|
|F12|f12|
|;|;|
|=|=|
|,|,|
|-|-|
|.|.|
|/|/|
|`|`|
|[|[|
|\\|\\|
|]|]|
|"|'|
|~|`|
|!|1|
|@|2|
|#|3|
|$|4|
|%|5|
|^|6|
|&|7|
|*|8|
|(|9|
|)|0|
|_|-|
|+|=|
|:|;|
|\<|,|
|\>|.|
|?|/|
|||\\|

### Type definition

```ts
type ValidKey = "BACKSPACE" |
    "TAB" |
    "ENTER" |
    "SHIFT" |
    "CTRL" |
    "ALT" |
    "CAPS" |
    "ESCAPE" |
    "SPACE" |
    "PAGE_UP" |
    "PAGE_DOWN" |
    "END" |
    "HOME" |
    "ARROW_LEFT" |
    "ARROW_UP" |
    "ARROW_RIGHT" |
    "ARROW_DOWN" |
    "INSERT" |
    "DELETE" |
    "A" |
    "B" |
    "C" |
    "D" |
    "E" |
    "F" |
    "G" |
    "H" |
    "I" |
    "J" |
    "K" |
    "L" |
    "M" |
    "N" |
    "O" |
    "P" |
    "Q" |
    "R" |
    "S" |
    "T" |
    "U" |
    "V" |
    "W" |
    "X" |
    "Y" |
    "Z" |
    "WINDOW_LEFT" |
    "WINDOW_RIGHT" |
    "SELECT" |
    "NUMPAD_0" |
    "NUMPAD_1" |
    "NUMPAD_2" |
    "NUMPAD_3" |
    "NUMPAD_4" |
    "NUMPAD_5" |
    "NUMPAD_6" |
    "NUMPAD_7" |
    "NUMPAD_8" |
    "NUMPAD_9" |
    "MULTIPLY" |
    "ADD" |
    "SUBTRACT" |
    "DECIMAL_POINT" |
    "DIVIDE" |
    "F1" |
    "F2" |
    "F3" |
    "F4" |
    "F5" |
    "F6" |
    "F7" |
    "F8" |
    "F9" |
    "F10" |
    "F11" |
    "F12" |
    ";" |
    "=" |
    " |
    " |
    "-" |
    "." |
    "/" |
    "`" |
    "[" |
    "\\" |
    "]" |
    "\"" |
    "~" |
    "!" |
    "@" |
    "#" |
    "$" |
    "%" |
    "^" |
    "&" |
    "*" |
    "(" |
    ")" |
    "_" |
    "+" |
    ":" |
    "<" |
    ">" |
    "?" |
    "|";
```


=== docs/development/api-wrapper/types/metadata.md ===
---
title: Metadata
description: Type of metadata object.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type Metadata = Partial<Record<string, string>>;
```

An example of metadata object, **not** a type definition, may not include all properties.

```ts
type Metadata = {
    actions.skipping_next_past_track: string;
    actions.skipping_prev_past_track: string;
    added_at: `${bigint}`;
    album_artist_name: string;
    album_disc_count: `${number}`;
    album_disc_number: `${number}`;
    album_title: string;
    album_track_count: `${number}`;
    album_track_number: `${number}`;
    album_uri: string;
    artist_name: string;
    artist_uri: string;
    // URL
    canvas.artist.avatar: string;
    canvas.artist.name: string;
    canvas.artist.uri: string;
    canvas.canvasUri: string;
    canvas.entityUri: string;
    canvas.explicit: "true" | "false";
    canvas.fileId: string;
    canvas.id: string;
    canvas.type: string;
    canvas.uploadedBy: string;
    // URL
    canvas.url: string;
    collection.can_add: "true" | "false";
    collection.can_ban: "true" | "false";
    collection.in_collection: "true" | "false";
    collection.is_banned: "true" | "false";
    context_uri: string;
    duration: `${bigint}`;
    entity_uri: string;
    has_lyrics: "true" | "false";
    // Internal URL paths, not URLs
    image_large_url: string;
    image_small_url: string;
    image_url: string;
    image_xlarge_url: string;
    interaction_id: string;
    iteration: `${number}`;
    marked_for_download:  "true" | "false";
    page_instance_id: string;
    popularity: `${number}`;
    title: string;
    track_player: string;
}
```


=== docs/development/api-wrapper/types/panel/panel-props.md ===
---
title: PanelProps
description: Properties that are used by the `registerPanel` function.
---

```ts
type PanelProps = {
    label?: string;
    children: React.ReactNode;
    isCustom?: boolean;
    style?: React.CSSProperties;
    wrapperClassname?: string;
    headerClassname?: string;
    headerVariant?: Variant;
    headerSemanticColor?: SemanticColor;
    headerLink?: string;
    headerActions?: React.ReactNode;
    headerOnClose?: () => void;
    headerPreventDefaultClose?: boolean;
    headerOnBack?: (event: React.MouseEvent<HTMLButtonElement>) => void;
};
```

| Property | Type | Description |
| --- | --- | --- |
| `label` | `string` &#124; `undefined` | Label of the Panel. |
| `children` | `React.ReactNode` | Children to render inside the Panel.<br />Must be a React Component. |
| `isCustom` | `boolean` &#124; `undefined` | Determine if the children passed is a custom Panel.<br />If true, the children will be rendered as is.<br />**Note**: All passed props except `children` will be ignored if enabled. |
| `style` | `React.CSSProperties` &#124; `undefined` | Inline styles to apply to the Panel skeleton. |
| `wrapperClassname` | `string` &#124; `undefined` | Additional class name to apply to the Panel content wrapper. |
| `headerClassname` | `string` &#124; `undefined` | Additional class name to apply to the Panel header. |
| `headerVariant` | [`Variant`](/docs/development/api-wrapper/types/variant) &#124; `undefined` | Font variant for the Panel header title. |
| `headerSemanticColor` | [`SemanticColor`](/docs/development/api-wrapper/types/semantic-color) &#124; `undefined` | Semantic color name for the Panel header title. |
| `headerLink` | `string` &#124; `undefined` | Href for the header link.<br />Can be either a URI, a path within the app, or a URL for an external link. |
| `headerActions` | `React.ReactNode` &#124; `undefined` | Additional actions to render in the header.<br />Will be rendered next to the close button. |
| `headerOnClose` | `() => void` &#124; `undefined` | Function to call when clicking on the header close button.<br />Called before the panel is closed. |
| `headerPreventDefaultClose` | `boolean` &#124; `undefined` | Prevent the panel from closing when clicking on the header close button. |
| `headerOnBack` | `(event: React.MouseEvent<HTMLButtonElement>) => void` &#124; `undefined` | Function to call when clicking on the header back button.<br />If not provided, the back button will not be rendered. |

=== docs/development/api-wrapper/types/player-state.md ===
---
title: PlayerState
description: PlayerState type definition.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type PlayerState = {
    timestamp: number;
    context_uri: string;
    context_url: string;
    context_restrictions: Record<string, string[]>;
    index?: {
        page: number;
        track: number;
    };
    track?: ProvidedTrack;
    playback_id?: string;
    playback_quality?: {
        bitrate_level: string;
        hifi_status?: string;
        strategy?: string;
        target_bitrate_available?: boolean;
        target_bitrate_level?: string;
    }
    playback_speed?: number;
    position_as_of_timestamp: number;
    duration: number;
    is_playing: boolean;
    is_paused: boolean;
    is_buffering: boolean;
    play_origin: {
        feature_identifier: string;
        feature_version: string;
        view_uri?: string;
        external_referrer?: string;
        referrer_identifier?: string;
        device_identifier?: string;
    };
    options: {
        shuffling_context?: boolean;
        repeating_context?: boolean;
        repeating_track?: boolean;
    };
    restrictions: Record<string, string[]>;
    suppressions: {
        providers: string[];
    };
    debug?: {
        log: string[];
    };
    prev_tracks?: ProvidedTrack[];
    next_tracks?: ProvidedTrack[];
    context_metadata: Metadata;
    page_metadata: Metadata;
    session_id: string;
    queue_revision?: string;
};
```

| Property | Type | Description |
| --- | --- | --- |
| `timestamp` | `number` | Timestamp. |
| `context_uri` | `string` | Context URI from which the track was played. |
| `context_url` | `string` | Context internal URL. |
| `context_restrictions` | `Record<string, string[]>` | Context restrictions. |
| `index` | `object` &#124; `undefined` | Track index. |
| `track` | [`ProvidedTrack`](/docs/development/api-wrapper/types/provided-track) &#124; `undefined` | Current track. |
| `playback_id` | `string` &#124; `undefined` | Playback ID. |
| `playback_quality` | `object` &#124; `undefined` | Playback quality. |
| `playback_speed` | `number` &#124; `undefined` | Playback speed. |
| `position_as_of_timestamp` | `number` | Position as of timestamp. Relative to the track's start. |
| `duration` | `number` | Track duration. |
| `is_playing` | `boolean` | Whether the track is playing. |
| `is_paused` | `boolean` | Whether the track is paused. |
| `is_buffering` | `boolean` | Whether the track is buffering. |
| `play_origin` | `object` | Play origin (client info). |
| `options` | `object` | Repeat and shuffle state. |
| `restrictions` | `Record<string, string[]>` | Restrictions. |
| `suppressions` | `object` | Suppressions from providers. |
| `debug` | `object` &#124; `undefined` | Debug info. |
| `prev_tracks` | [`ProvidedTrack[]`](/docs/development/api-wrapper/types/provided-track) &#124; `undefined` | Previous tracks. |
| `next_tracks` | [`ProvidedTrack[]`](/docs/development/api-wrapper/types/provided-track) &#124; `undefined` | Next tracks. |
| `context_metadata` | [`Metadata`](/docs/development/api-wrapper/types/metadata) | Context metadata. |
| `page_metadata` | [`Metadata`](/docs/development/api-wrapper/types/metadata) | Page metadata. |
| `session_id` | `string` | Session ID. |
| `queue_revision` | `string` &#124; `undefined` | Queue revision. |


=== docs/development/api-wrapper/types/provided-track.md ===
---
title: ProvidedTrack
description: ProvidedTrack type definition.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type ProvidedTrack = ContextTrack & {
    removed?: string[];
    blocked?: string[];
    provider?: string;
};
```

Extends [`ContextTrack`](/docs/development/api-wrapper/types/context-track).

| Property | Type | Description |
| --- | --- | --- |
| `removed` | `string[]` &#124; `undefined` | List of removed providers. |
| `blocked` | `string[]` &#124; `undefined` | List of blocked providers. |
| `provider` | `string` | Current provider, eg. from `context` |


=== docs/development/api-wrapper/types/react-component/confirm-dialog-props.md ===
---
title: ConfirmDialogProps
description: Type definition for props of ReactComponent.ConfirmDialog
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

The `ConfirmDialogProps` object is used to create a confirm dialog.

```ts
type ConfirmDialogProps = {
    isOpen?: boolean;
    allowHTML?: boolean;
    titleText: string;
    descriptionText?: string;
    confirmText?: string;
    cancelText?: string;
    confirmLabel?: string;
    onConfirm?: (event: React.MouseEvent<HTMLButtonElement>) => void;
    onClose?: (event: React.MouseEvent<HTMLButtonElement>) => void;
    onOutside?: (event: React.MouseEvent<HTMLButtonElement>) => void;
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| `isOpen` | `boolean` &#124; `undefined` | Boolean to determine if the dialog should be opened. Defaults to `true` |
| `allowHTML` | `boolean` &#124; `undefined` | Whether to allow inline HTML in component text. Defaults to `false` |
| `titleText` | `string` | Dialog title. Can be inline HTML if `allowHTML` is true |
| `descriptionText` | `string` &#124; `undefined` | Dialog description. Can be inline HTML if `allowHTML` is true |
| `confirmText` | `string` &#124; `undefined` | Confirm button text |
| `cancelText` | `string` &#124; `undefined` | Cancel button text |
| `confirmLabel` | `string` &#124; `undefined` | Confirm button `aria-label` |
| `onConfirm` | `(event: React.MouseEvent<HTMLButtonElement>) => void` &#124; `undefined` | Function to run when confirm button is clicked.<br />The dialog does not close automatically, a handler must be included. |
| `onClose` | `(event: React.MouseEvent<HTMLButtonElement>) => void` &#124; `undefined` | Function to run when cancel button is clicked.<br />The dialog does not close automatically, a handler must be included. |
| `onOutside` | `(event: React.MouseEvent<HTMLButtonElement>) => void` &#124; `undefined` | Function to run when dialog is clicked outside of.<br />By default, this will run `onClose`.<br />A handler must be included to close the dialog. |


=== docs/development/api-wrapper/types/react-component/context-menu-props.md ===
---
title: ContextMenuProps
description: Type definition for props of ReactComponent.ContextMenu
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

The `ContextMenuProps` object is used to create a context menu.

```ts
type ContextMenuProps = {
    renderInline?: boolean;
    trigger?: 'click' | 'right-click';
    action?: 'toggle' | 'open';
    placement?: 'top' | 'top-start' | 'top-end' | 'right' | 'right-start' | 'right-end' | 'bottom' | 'bottom-start' | 'bottom-end' | 'left' | 'left-start' | 'left-end';
    offset?: [number, number];
    preventScrollingWhileOpen?: boolean;
    menu: typeof Spicetify.ReactComponent.Menu;
    children: Element | ((isOpen?: boolean, handleContextMenu?: (e: MouseEvent) => void, ref?: (e: Element) => void) => Element);
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| `renderInline` | `boolean` | Decide whether to use the global singleton context menu (rendered in `<body>`) or a new inline context menu (rendered in a sibling element to `children`) |
| `trigger` | `'click'` &#124; `'right-click'` | Determines what will trigger the context menu. For example, a click, or a right-click |
| `action` | `'toggle'` &#124; `'open'` | Determines if the context menu should open or toggle when triggered |
| `placement` | `'top'` &#124; `'top-start'` &#124; `'top-end'` &#124; `'right'` &#124; `'right-start'` &#124; `'right-end'` &#124; `'bottom'` &#124; `'bottom-start'` &#124; `'bottom-end'` &#124; `'left'` &#124; `'left-start'` &#124; `'left-end'` | The preferred placement of the context menu when it opens. Relative to trigger element. |
| `offset` | `[number, number]` | The x and y offset distances at which the context menu should open. Relative to trigger element and `position`. |
| `preventScrollingWhileOpen` | `boolean` | Will stop the client from scrolling while the context menu is open |
| `menu` | `typeof Spicetify.ReactComponent.Menu` | The menu UI to render inside of the context menu. |


=== docs/development/api-wrapper/types/react-component/icon-component-props.md ===
---
title: IconComponentProps
description: Type definition for props of ReactComponent.IconComponent.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

The `IconComponentProps` object is used to create an icon component.

```ts
type IconComponentProps = {
    iconSize?: number;
    color?: string;
    semanticColor?: SemantiColor;
    title?: string;
    titleId?: string;
    desc?: string;
    descId?: string;
    autoMirror?: boolean;
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| iconSize | `number` &#124; `undefined` | Icon size |
| color | `string` &#124; `undefined` | Icon color. Might not be used by component |
| semanticColor | [`SemanticColor`](../semantic-color) &#124; `undefined` | Semantic color name. Matches color variables used in xpui |
| title | `string` &#124; `undefined` | Icon title |
| titleId | `string` &#124; `undefined` | Title ID (internal) |
| desc | `string` &#124; `undefined` | Icon description |
| descId | `string` &#124; `undefined` | Description ID (internal) |
| autoMirror | `boolean` &#124; `undefined` | Whether the icon can be auto mirrored |


=== docs/development/api-wrapper/types/react-component/menu-item-props.md ===
---
title: MenuItemProps
description: Type definition for props of ReactComponent.MenuItem.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

The `MenuItemProps` object is used to create a menu item.

```ts
type MenuItemProps = {
    onClick?: React.MouseEventHandler<HTMLButtonElement>;
    disabled?: boolean;
    divider?: 'before' | 'after' | 'both';
    icon?: React.ReactNode;
    leadingIcon?: React.ReactNode;
    trailingIcon?: React.ReactNode;
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| `onClick` | `React.MouseEventHandler<HTMLButtonElement>` &#124; `undefined` | Function that runs when `MenuItem` is clicked |
| `disabled` | `boolean` &#124; `undefined` | Indicates if `MenuItem` is disabled. Disabled items will not cause the `Menu` to close when clicked. |
| `divider` | `'before' &#124; 'after' &#124; 'both'` &#124; `undefined` | Indicate that a divider line should be added `before` or `after` this `MenuItem` |
| `icon` | `React.ReactNode` &#124; `undefined` | React component icon that will be rendered at the end of the `MenuItem`. **Deprecated**: Since Spotify `1.2.8`. Use `leadingIcon` or `trailingIcon` instead |
| `leadingIcon` | `React.ReactNode` &#124; `undefined` | React component icon that will be rendered at the start of the `MenuItem`. **Since Spotify `1.2.8`** |
| `trailingIcon` | `React.ReactNode` &#124; `undefined` | React component icon that will be rendered at the start of the `MenuItem`. **Since Spotify `1.2.8`** |


=== docs/development/api-wrapper/types/react-component/menu-props.md ===
---
title: MenuProps
description: Type definition for props of ReactComponent.Menu.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

The `MenuProps` object is used to create a menu.

```ts
type MenuProps = {
    onClose?: () => void;
    getInitialFocusElement?: (el: HTMLElement | null) => HTMLElement | undefined | null;
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| onClose | `() => void` &#124; `undefined` | Function that is called when the menu is closed |
| getInitialFocusElement | `(el: HTMLElement &#124; null) => HTMLElement &#124; undefined &#124; null` &#124; `undefined` | Function that provides the element that focus should jump to when the menu is opened |


=== docs/development/api-wrapper/types/react-component/panel-content-props.md ===
---
title: PanelContentProps
description: Type definition for props of ReactComponent.PanelContent.
---

The `PanelContentProps` object is used to render a panel content wrapper.

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type PanelContentProps = {
    className?: string;
    children?: React.ReactNode;
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| className | `string` &#124; `undefined` | Additional class name to apply to the panel. |
| children | `React.ReactNode` &#124; `undefined` | Children to render inside the panel. |

=== docs/development/api-wrapper/types/react-component/panel-header-props.md ===
---
title: PanelHeaderProps
description: Type definition for props of ReactComponent.PanelHeader.
---

The `PanelHeaderProps` object is used to render a panel header.

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type PanelHeaderProps = {
    link?: string;
    title?: string;
    panel: number;
    isAdvert?: boolean;
    actions?: React.ReactNode;
    onClose?: () => void;
    preventDefaultClose?: boolean;
    onBack?: (event: React.MouseEvent<HTMLButtonElement>) => void;
    titleVariant?: Variant;
    titleSemanticColor?: SemanticColor;
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| link | `string` &#124; `undefined` | Href for the header link.<br />Can be either a URI, a path within the app, or a URL for an external link. |
| title | `string` &#124; `undefined` | Title of the header. |
| panel | `number` | Panel ID. Used to toggle panel open/closed state. |
| isAdvert | `boolean` &#124; `undefined` | Whether or not the panel contains advertisements. Defaults to `false` |
| actions | `React.ReactNode` &#124; `undefined` | Actions to render in the header. |
| onClose | `() => void` &#124; `undefined` | Function to call when clicking on the close button.<br />Called before the panel is closed. |
| preventDefaultClose | `boolean` &#124; `undefined` | Prevent the panel from closing when clicking on the header close button. Defaults to `false` |
| onBack | `(event: React.MouseEvent<HTMLButtonElement>) => void` &#124; `undefined` | Function to call when clicking on the header back button.<br />If not provided, the back button will not be rendered. |
| titleVariant | [`Variant`](/docs/development/api-wrapper/types/variant) &#124; `undefined` | Font variant for the header title. Defaults to `"balladBold"` |
| titleSemanticColor | [`SemanticColor`](/docs/development/api-wrapper/types/semantic-color) &#124; `undefined` | Semantic color name for the header title. Defaults to `"textBase"` |

=== docs/development/api-wrapper/types/react-component/panel-skeleton-props.md ===
---
title: PanelSkeletonProps
description: Type definition for props of ReactComponent.PanelSkeleton.
---

The `PanelSkeletonProps` object is used to render a panel skeleton.

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type PanelSkeletonProps = {
    label?: string;
    itemUri?: string;
    className?: string;
    style?: React.CSSProperties;
    children?: React.ReactNode;
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| label | `string` &#124; `undefined` | Aria label for the panel. Does not set the panel header content. |
| itemUri | `string` &#124; `undefined` | Item URI of the panel. Used as reference for Spotify's internal Event Factory. |
| className | `string` &#124; `undefined` | Additional class name to apply to the panel. **Deprecated**: Since Spotify `1.2.12` |
| style | `React.CSSProperties` &#124; `undefined` | Additional styles to apply to the panel. |
| children | `React.ReactNode` &#124; `undefined` | Children to render inside the panel. |

=== docs/development/api-wrapper/types/react-component/slider-props.md ===
---
title: SliderProps
description: Type definition for props of ReactComponent.PanelSkeleton.
---

The `SliderProps` object is used to render a slider.

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type SliderProps = {
    value: number;
    max: number;
    step: number;
    labelText?: string;
    isInteractive?: boolean;
    forceActiveStyles?: boolean;
    onDragStart: (value: number) => void;
    onDragMove: (value: number) => void;
    onDragEnd: (value: number) => void;
    onStepForward?: (value: number) => void;
    onStepBackward?: (value: number) => void;
}
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| value | `number` | The current value of the slider. |
| max | `number` | The maximum value the slider can have. |
| step | `number` | The increment/decrement value when the slider is moved. |
| labelText | `string` &#124; `undefined` | The label text displayed for the slider. |
| isInteractive | `boolean` &#124; `undefined` | Determines if the slider is interactive. |
| forceActiveStyles | `boolean` &#124; `undefined` | Forces the active styles regardless of interaction state. |
| onDragStart | `(value: number) => void` | Callback function when dragging starts. |
| onDragMove | `(value: number) => void` | Callback function when the slider is being dragged. |
| onDragEnd | `(value: number) => void` | Callback function when dragging ends. |
| onStepForward | `(value: number) => void` &#124; `undefined` | Callback function when the slider steps forward. **Deprecated.** |
| onStepBackward | `(value: number) => void` &#124; `undefined` | Callback function when the slider steps backward. **Deprecated.** |


=== docs/development/api-wrapper/types/react-component/text-component-props.md ===
---
title: TextComponentProps
description: Type definition for props of ReactComponent.TextComponent.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

The `TextComponentProps` object is used to create a text component.

```ts
type TextComponentProps = {
    color?: string;
    semanticColor?: SemanticColor;
    variant?: Variant;
    paddingBottom?: string;
    weight?: 'book' | 'bold' | 'black';
};
```

#### Properties

| Property | Type | Description |
| -------- | ---- | ----------- |
| color | `string` &#124; `undefined` | Text color. Might not be used by component. |
| semanticColor | [`SemanticColor`](../semantic-color) &#124; `undefined` | Semantic color name. Matches color variables used in xpui |
| variant | [`Variant`](../variant) &#124; `undefined` | Font variant |
| paddingBottom | `string` &#124; `undefined` | Bottom padding size |
| weight | `'book'` &#124; `'bold'` &#124; `'black'` &#124; `undefined` | Font weight |


=== docs/development/api-wrapper/types/react-component/toggle-props.md ===
---
title: ToggleProps
description: Type definition for props of ReactComponent.Toggle.
---

The `ToggleProps` object is used to render a toggle.

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type ToggleProps = {
    value: boolean;
    disabled?: boolean;
    onSelected: (value: boolean) => void;
    id?: string;
    className?: string;
}
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| value | `boolean` | The current state of the toggle. `true` means it's on, `false` means it's off. |
| disabled | `boolean` &#124; `undefined` | Determines if the toggle is disabled. If `true`, the toggle is not interactive. |
| onSelected | `(value: boolean) => void` | Callback function that is called when the toggle is clicked. The function receives the new state of the toggle. |
| id | `string` &#124; `undefined` | The ID for the toggle, useful for associating with a label for accessibility. |
| className | `string` &#124; `undefined` | Additional CSS class name to apply to the toggle. |


=== docs/development/api-wrapper/types/react-component/tooltip-props.md ===
---
title: TooltipProps
description: Type definition for props of ReactComponent.TooltipWrapper.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

The `TooltipProps` object is used to create a tooltip.

```ts
type TooltipProps = {
    label: string;
    children: React.ReactNode;
    renderInline?: boolean;
    showDelay?: number;
    disabled?: boolean;
    placement?: 'top' | 'top-start' | 'top-end' | 'right' | 'right-start' | 'right-end' | 'bottom' | 'bottom-start' | 'bottom-end' | 'left' | 'left-start' | 'left-end';
    labelClassName?: string;
};
```

#### Properties

| Property | Type | Description |
| :--- | :--- | :--- |
| `label` | `string` | Label to display in the tooltip |
| `children` | `React.ReactNode` | The child element that the tooltip will be attached to and will display when hovered over |
| `renderInline` | `boolean` | Decide whether to use the global singleton tooltip (rendered in `<body>`) or a new inline tooltip (rendered in a sibling element to `children`) |
| `showDelay` | `number` | Delay in milliseconds before the tooltip is displayed after the user hovers over the child element |
| `disabled` | `boolean` | Determine whether the tooltip should be displayed |
| `placement` | `'top'` &#124; `'top-start'` &#124; `'top-end'` &#124; `'right'` &#124; `'right-start'` &#124; `'right-end'` &#124; `'bottom'` &#124; `'bottom-start'` &#124; `'bottom-end'` &#124; `'left'` &#124; `'left-start'` &#124; `'left-end'` | The preferred placement of the context menu when it opens. Relative to trigger element. |
| `labelClassName` | `string` | Class name to apply to the tooltip |


=== docs/development/api-wrapper/types/semantic-color.md ===
---
title: SemanticColor
description: Semantic color names used in the Spotify app.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type SemanticColor = "textBase" |
    "textSubdued" |
    "textBrightAccent" |
    "textNegative" |
    "textWarning" |
    "textPositive" |
    "textAnnouncement" |
    "essentialBase" |
    "essentialSubdued" |
    "essentialBrightAccent" |
    "essentialNegative" |
    "essentialWarning" |
    "essentialPositive" |
    "essentialAnnouncement" |
    "decorativeBase" |
    "decorativeSubdued" |
    "backgroundBase" |
    "backgroundHighlight" |
    "backgroundPress" |
    "backgroundElevatedBase" |
    "backgroundElevatedHighlight" |
    "backgroundElevatedPress" |
    "backgroundTintedBase" |
    "backgroundTintedHighlight" |
    "backgroundTintedPress" |
    "backgroundUnsafeForSmallTextBase" |
    "backgroundUnsafeForSmallTextHighlight" |
    "backgroundUnsafeForSmallTextPress";
```


=== docs/development/api-wrapper/types/svgicon.md ===
---
title: SVGIcon
description: SVGIcon type definition.
---

```ts
type SVGIcon = "album" |
    "artist" |
    "block" |
    "brightness" |
    "car" |
    "chart-down" |
    "chart-up" |
    "check" |
    "check-alt-fill" |
    "chevron-left" |
    "chevron-right" |
    "chromecast-disconnected" |
    "clock" |
    "collaborative" |
    "computer" |
    "copy" |
    "download" |
    "downloaded" |
    "edit" |
    "enhance" |
    "exclamation-circle" |
    "external-link" |
    "facebook" |
    "follow" |
    "fullscreen" |
    "gamepad" |
    "grid-view" |
    "heart" |
    "heart-active" |
    "instagram" |
    "laptop" |
    "library" |
    "list-view" |
    "location" |
    "locked" |
    "locked-active" |
    "lyrics" |
    "menu" |
    "minimize" |
    "minus" |
    "more" |
    "new-spotify-connect" |
    "offline" |
    "pause" |
    "phone" |
    "play" |
    "playlist" |
    "playlist-folder" |
    "plus-alt" |
    "plus2px" |
    "podcasts" |
    "projector" |
    "queue" |
    "repeat" |
    "repeat-once" |
    "search" |
    "search-active" |
    "shuffle" |
    "skip-back" |
    "skip-back15" |
    "skip-forward" |
    "skip-forward15" |
    "soundbetter" |
    "speaker" |
    "spotify" |
    "subtitles" |
    "tablet" |
    "ticket" |
    "twitter" |
    "visualizer" |
    "voice" |
    "volume" |
    "volume-off" |
    "volume-one-wave" |
    "volume-two-wave" |
    "watch" |
    "x";
```

### Examples
![svg-examples](/images/spicetify-svg-examples.png)


=== docs/development/api-wrapper/types/uri/type.md ===
---
title: Type
description: Type of a URI.
---

Type of a URI.

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
class URI {
    static Type: {
        AD: string;
        ALBUM: string;
        GENRE: string;
        QUEUE: string;
        APPLICATION: string;
        ARTIST: string;
        ARTIST_TOPLIST: string;
        ARTIST_CONCERTS: string;
        AUDIO_FILE: string;
        COLLECTION: string;
        COLLECTION_ALBUM: string;
        COLLECTION_ARTIST: string;
        COLLECTION_MISSING_ALBUM: string;
        COLLECTION_TRACK_LIST: string;
        CONCERT: string;
        CONTEXT_GROUP: string;
        DAILY_MIX: string;
        EMPTY: string;
        EPISODE: string;
        /** URI particle; not an actual URI. */
        FACEBOOK: string;
        FOLDER: string;
        FOLLOWERS: string;
        FOLLOWING: string;
        IMAGE: string;
        INBOX: string;
        INTERRUPTION: string;
        LIBRARY: string;
        LIVE: string;
        ROOM: string;
        EXPRESSION: string;
        LOCAL: string;
        LOCAL_TRACK: string;
        LOCAL_ALBUM: string;
        LOCAL_ARTIST: string;
        MERCH: string;
        MOSAIC: string;
        PLAYLIST: string;
        PLAYLIST_V2: string;
        PRERELEASE: string;
        PROFILE: string;
        PUBLISHED_ROOTLIST: string;
        RADIO: string;
        ROOTLIST: string;
        SEARCH: string;
        SHOW: string;
        SOCIAL_SESSION: string;
        SPECIAL: string;
        STARRED: string;
        STATION: string;
        TEMP_PLAYLIST: string;
        TOPLIST: string;
        TRACK: string;
        TRACKSET: string;
        USER_TOPLIST: string;
        USER_TOP_TRACKS: string;
        UNKNOWN: string;
        MEDIA: string;
        QUESTION: string;
        POLL: string;
    };
};
```


=== docs/development/api-wrapper/types/uri/validation-functions.md ===
---
title: Validation functions
description: Functions that can be used to validate a URI type.
---

```ts
class URI {
  static isAd(uri: any): boolean;
  static isAlbum(uri: any): boolean;
  static isGenre(uri: any): boolean;
  static isQueue(uri: any): boolean;
  static isApplication(uri: any): boolean;
  static isArtist(uri: any): boolean;
  static isArtistToplist(uri: any): boolean;
  static isArtistConcerts(uri: any): boolean;
  static isAudioFile(uri: any): boolean;
  static isCollection(uri: any): boolean;
  static isCollectionAlbum(uri: any): boolean;
  static isCollectionArtist(uri: any): boolean;
  static isCollectionMissingAlbum(uri: any): boolean;
  static isCollectionTrackList(uri: any): boolean;
  static isConcert(uri: any): boolean;
  static isContextGroup(uri: any): boolean;
  static isDailyMix(uri: any): boolean;
  static isEmpty(uri: any): boolean;
  static isEpisode(uri: any): boolean;
  static isFacebook(uri: any): boolean;
  static isFolder(uri: any): boolean;
  static isFollowers(uri: any): boolean;
  static isFollowing(uri: any): boolean;
  static isImage(uri: any): boolean;
  static isInbox(uri: any): boolean;
  static isInterruption(uri: any): boolean;
  static isLibrary(uri: any): boolean;
  static isLive(uri: any): boolean;
  static isRoom(uri: any): boolean;
  static isExpression(uri: any): boolean;
  static isLocal(uri: any): boolean;
  static isLocalTrack(uri: any): boolean;
  static isLocalAlbum(uri: any): boolean;
  static isLocalArtist(uri: any): boolean;
  static isMerch(uri: any): boolean;
  static isMosaic(uri: any): boolean;
  static isPlaylist(uri: any): boolean;
  static isPlaylistV2(uri: any): boolean;
  static isPrerelease(uri: any): boolean;
  static isProfile(uri: any): boolean;
  static isPublishedRootlist(uri: any): boolean;
  static isRadio(uri: any): boolean;
  static isRootlist(uri: any): boolean;
  static isSearch(uri: any): boolean;
  static isShow(uri: any): boolean;
  static isSocialSession(uri: any): boolean;
  static isSpecial(uri: any): boolean;
  static isStarred(uri: any): boolean;
  static isStation(uri: any): boolean;
  static isTempPlaylist(uri: any): boolean;
  static isToplist(uri: any): boolean;
  static isTrack(uri: any): boolean;
  static isTrackset(uri: any): boolean;
  static isUserToplist(uri: any): boolean;
  static isUserTopTracks(uri: any): boolean;
  static isUnknown(uri: any): boolean;
  static isMedia(uri: any): boolean;
  static isQuestion(uri: any): boolean;
  static isPoll(uri: any): boolean;
  static isPlaylistV1OrV2(uri: any): boolean;
}
```


=== docs/development/api-wrapper/types/variant.md ===
---
title: Variant
description: Font variants used in the Spotify app.
---

:::note

This type is deducted from Spotify's internal usage. It may not be accurate and may change in the future.

:::

```ts
type Variant = "bass" |
    "forte" |
    "brio" |
    "altoBrio" |
    "alto" |
    "canon" |
    "celloCanon" |
    "cello" |
    "ballad" |
    "balladBold" |
    "viola" |
    "violaBold" |
    "mesto" |
    "mestoBold" |
    "metronome" |
    "finale" |
    "finaleBold" |
    "minuet" |
    "minuetBold";
```


=== docs/development/compiling.md ===
---
title: Compiling
description: 🧰 Compiling Spicetify.
---

### Requirements

- [Go](https://golang.org/dl/)

Clone repo and download dependencies:

```bash
cd $HOME
mkdir spicetify
cd spicetify
git clone https://github.com/spicetify/cli
```

### Build

#### Windows

```powershell
cd $HOME\spicetify\cli
go build -o spicetify.exe
```

#### Linux and MacOS

```bash
cd ~/spicetify/cli
go build -o spicetify
```


=== docs/development/custom-apps.md ===
---
title: Custom Apps
description: 🔧 Creating Custom Apps.
---

## App Structure

```text
~/.config/spicetify/CustomApps/   # Linux/macOS
%appdata%\spicetify\CustomApps\   # Windows
└── my-app/
    ├── index.js                  # Required: main entry with render()
    ├── manifest.json             # Required: app metadata and icons
    ├── style.css                 # Optional: custom styles
    └── src/                      # Optional: additional JS files
        ├── Component.js
        └── ...
```

## How to Start

- Make a new folder for your custom app in your [CustomApps folder](/docs/customization/custom-apps). You'll install it like any other custom app.
- Create an `index.js` and a `manifest.json` inside that folder.
- The `index.js` file is the main file for the custom app.
- The manifest includes some important information to make the custom app work.

## Manifest file

- Your custom app needs a `manifest.json` file in the root folder with the following keys:
- `name`: The name of the custom app.
- `icon`: The escaped SVG markup for the sidebar icon.
- `active-icon`: The escaped SVG markup for the active status of the sidebar icon (when your custom app is open).
- `subfiles`: You can optionally include other JS files. These files will be concatenated together in the order defined here. Any variables you declare in the main `index.js`, or any subfiles will be accessible from all. This is useful for organizational purposes for more complex custom apps.
- `subfiles_extension`: You can optionally include one or more extensions with your custom app. These are treated as regular extensions, and will run when Spotify starts.

_Note: The `subfiles` can be in nested folders, while any `subfiles_extension` can not._

```json
{
  "name": "My Custom App",
  "icon": "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 576 512\"><path fill=\"currentColor\" d=\"M504.717 320H211.572l6.545 32h268.418c15.401 0 26.816 14.301 23.403 29.319l-5.517 24.276C523.112 414.668 536 433.828 536 456c0 31.202-25.519 56.444-56.824 55.994-29.823-.429-54.35-24.631-55.155-54.447-.44-16.287 6.085-31.049 16.803-41.548H231.176C241.553 426.165 248 440.326 248 456c0 31.813-26.528 57.431-58.67 55.938-28.54-1.325-51.751-24.385-53.251-52.917-1.158-22.034 10.436-41.455 28.051-51.586L93.883 64H24C10.745 64 0 53.255 0 40V24C0 10.745 10.745 0 24 0h102.529c11.401 0 21.228 8.021 23.513 19.19L159.208 64H551.99c15.401 0 26.816 14.301 23.403 29.319l-47.273 208C525.637 312.246 515.923 320 504.717 320zM403.029 192H360v-60c0-6.627-5.373-12-12-12h-24c-6.627 0-12 5.373-12 12v60h-43.029c-10.691 0-16.045 12.926-8.485 20.485l67.029 67.029c4.686 4.686 12.284 4.686 16.971 0l67.029-67.029c7.559-7.559 2.205-20.485-8.486-20.485z\"></path></svg>",
  "active-icon": "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 576 512\"><path fill=\"currentColor\" d=\"M504.717 320H211.572l6.545 32h268.418c15.401 0 26.816 14.301 23.403 29.319l-5.517 24.276C523.112 414.668 536 433.828 536 456c0 31.202-25.519 56.444-56.824 55.994-29.823-.429-54.35-24.631-55.155-54.447-.44-16.287 6.085-31.049 16.803-41.548H231.176C241.553 426.165 248 440.326 248 456c0 31.813-26.528 57.431-58.67 55.938-28.54-1.325-51.751-24.385-53.251-52.917-1.158-22.034 10.436-41.455 28.051-51.586L93.883 64H24C10.745 64 0 53.255 0 40V24C0 10.745 10.745 0 24 0h102.529c11.401 0 21.228 8.021 23.513 19.19L159.208 64H551.99c15.401 0 26.816 14.301 23.403 29.319l-47.273 208C525.637 312.246 515.923 320 504.717 320zM403.029 192H360v-60c0-6.627-5.373-12-12-12h-24c-6.627 0-12 5.373-12 12v60h-43.029c-10.691 0-16.045 12.926-8.485 20.485l67.029 67.029c4.686 4.686 12.284 4.686 16.971 0l67.029-67.029c7.559-7.559 2.205-20.485-8.486-20.485z\"></path></svg>",
  "subfiles": ["src/Subfile.js", "src/Subfile2.js"],
  "subfiles_extension": ["my_extension.js"]
}
```

## Creating your index.js

Custom apps are written in [React](https://reactjs.org). You'll need to grab some React references from the `Spicetify` object, set up a component for your page, and return that component from a `render()` function in the main body of the file. Sadly, this does not support [`jsx`](https://reactjs.org/docs/introducing-jsx.html), so you must use [`react.createElement`](https://reactjs.org/docs/react-api.html#createelement).

Example:

```js
// Grab any variables you need
const react = Spicetify.React;
const reactDOM = Spicetify.ReactDOM;
const {
    URI,
    React: { useState, useEffect, useCallback },
    Platform: { History },
} = Spicetify;

// The main custom app render function. The component returned is what is rendered in Spotify.
function render() {
    return react.createElement(Grid, { title: "My Custom App" });
}

// Our main component
class Grid extends react.Component {
    constructor(props) {
        super(props);
        Object.assign(this, props);
        this.state = {
            foo: "bar",
            data: "etc"
        };
    }

    render() {
        return react.createElement("section", {
            className: "contentSpacing",
        },
        react.createElement("div", {
            className: "marketplace-header",
        }, react.createElement("h1", null, this.props.title),
        ),
        ), react.createElement("div", {
            id: "marketplace-grid",
            className: "main-gridContainer-gridContainer",
            "data-tab": CONFIG.activeTab,
            style: {
                "--minimumColumnWidth": "180px",
            },
        }, [...cardList]),
        react.createElement("footer", {
            style: {
                margin: "auto",
                textAlign: "center",
            },
        }, !this.state.endOfList && (this.state.rest ? react.createElement(LoadMoreIcon, { onClick: this.loadMore.bind(this) }) : react.createElement(LoadingIcon)),
        ), react.createElement(TopBarContent, {
            switchCallback: this.switchTo.bind(this),
            links: CONFIG.tabs,
            activeLink: CONFIG.activeTab,
        }));
    }
}
```

## Common questions:

### My custom app isn't running when Spotify starts

Your custom app will only run when it is clicked on the sidebar and its page loads. In order to run code on startup, you need to include a separate JS file as `subfiles_extension` in your [manifest](#manifest-file).

### My subfile extension can't read my variables from my custom app

Any subfile extensions are loaded separately from the main custom app, and do not have access to variables. You can use `localStorage` to save/load data between the two.

### How can I add a new sub page or path to my custom app?

You can use `Spicetify.Platform.History.push(...)` to navigate to a new page. This can be a standard Spotify page, or a custom page for your app. You can include any data you need in the `state` key.

```js
Spicetify.Platform.History.push({
  pathname: '/marketplace/readme',
  state: {
    data: {
      title: 'My sub page title',
      content: 'My sub page content',
    },
  },
});
```

In order to render a different page, you can check the `pathname` of the current page within `index.js`'s main render method, and render a different page component for different paths. The main path for your custom app will be the name of the folder (which is the same that needs to be used in the [`config-xpui.ini`](/docs/development/themes) configuration file).
In this example, if our `pathname` is "/marketplace/readme", we load the `ReadmePage` component, otherwise we load our main page component, `Grid`.

```js
function render() {
  const { location } = Spicetify.Platform.History;

  // If page state set to display readme, render it
  // (This location state data comes from your Spicetify.Platform.History.push() call
  if (location.pathname === '/marketplace/readme') {
    return react.createElement(ReadmePage, {
      title: 'Spicetify Marketplace - Readme',
      data: location.state.data,
    });
  } // Otherwise, render the main Grid
  else {
    return react.createElement(Grid, { title: 'Spicetify Marketplace' });
  }
}
```


=== docs/development/extensions.md ===
---
title: Extensions
description: Creating extensions for Spicetify.
---

Extensions are JavaScript files that run alongside Spotify's main code. They can add UI elements, modify behavior, integrate with external services, and more.

## Extension Structure

```text
~/.config/spicetify/Extensions/   # Linux/macOS
%appdata%\spicetify\Extensions\   # Windows
├── myExtension.js
├── anotherExtension.js
└── ...
```

Extensions are single JavaScript files. For complex extensions with multiple files, use [Spicetify Creator](/docs/development/spicetify-creator/the-basics).

## Getting Started

### Location

Place your extension file in the Extensions folder:

| Platform | Path |
|----------|------|
| **Windows** | `%appdata%\spicetify\Extensions\` |
| **Linux / macOS** | `~/.config/spicetify/Extensions/` |

### Minimal Extension

```js
// myExtension.js
(function myExtension() {
    // Wait for Spicetify to be ready
    if (!Spicetify.Player || !Spicetify.Platform) {
        setTimeout(myExtension, 100);
        return;
    }

    console.log("My extension loaded!");

    // Your code here
})();
```

### Testing Your Extension

1. Save your file to the Extensions folder
2. Enable it:

   ```bash
   spicetify config extensions myExtension.js
   spicetify apply
   ```

3. Open DevTools in Spotify to see console output:
   - **Windows/Linux**: <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd>
   - **macOS**: <kbd>Cmd</kbd> + <kbd>Option</kbd> + <kbd>I</kbd>


---

## The Spicetify API

Extensions access Spotify through the global `Spicetify` object. Key namespaces:

| Namespace | Purpose |
|-----------|---------|
| `Spicetify.Player` | Playback control (play, pause, skip, volume) |
| `Spicetify.Platform` | Core platform APIs (history, library, playlists) |
| `Spicetify.CosmosAsync` | HTTP requests to Spotify's internal API |
| `Spicetify.URI` | Parse and create Spotify URIs |
| `Spicetify.React` | React instance for UI components |
| `Spicetify.ReactDOM` | ReactDOM for rendering |
| `Spicetify.Topbar` | Add buttons to the top bar |
| `Spicetify.Playbar` | Add buttons to the player bar |
| `Spicetify.ContextMenu` | Add items to right-click menus |

See the **[API Wrapper Reference](/docs/development/api-wrapper)** for complete documentation.

---

## Common Patterns

### Waiting for Spicetify

Always wait for Spicetify to be ready before accessing its APIs:

```js
(function init() {
    if (!Spicetify.Player || !Spicetify.Platform) {
        setTimeout(init, 100);
        return;
    }

    main();
})();

function main() {
    // Safe to use Spicetify here
}
```

### Adding a Top Bar Button

```js
const button = new Spicetify.Topbar.Button(
    "My Button",
    `<svg>...</svg>`,  // SVG icon
    () => {
        console.log("Button clicked!");
    }
);
```

### Adding a Context Menu Item

```js
new Spicetify.ContextMenu.Item(
    "My Menu Item",
    (uris) => {
        console.log("Selected URIs:", uris);
    },
    (uris) => {
        // Return true to show this item
        return true;
    },
    `<svg>...</svg>`  // Optional icon
).register();
```

### Listening to Player Events

```js
Spicetify.Player.addEventListener("songchange", (event) => {
    const track = Spicetify.Player.data?.item;
    console.log("Now playing:", track?.name);
});
```

### Making API Requests

```js
// GET request
const response = await Spicetify.CosmosAsync.get(
    "https://api.spotify.com/v1/me/player"
);

// POST request
await Spicetify.CosmosAsync.post(
    "https://api.spotify.com/v1/me/player/play",
    { uris: ["spotify:track:..."] }
);
```

### Storing Data

```js
// Save
Spicetify.LocalStorage.set("myExtension:setting", "value");

// Load
const value = Spicetify.LocalStorage.get("myExtension:setting");
```

### Showing Notifications

```js
Spicetify.showNotification("Hello from my extension!");
```

---

## UI Development

### Using React

Spicetify exposes React for building UI:

```js
const { React, ReactDOM } = Spicetify;
const { useState, useEffect } = React;

function MyComponent() {
    const [count, setCount] = useState(0);

    return React.createElement("button", {
        onClick: () => setCount(c => c + 1)
    }, `Clicked ${count} times`);
}
```

### Using Spotify's Components

Access Spotify's internal React components:

```js
const { Button, Toggle } = Spicetify.ReactComponent;

// Use in your components
React.createElement(Button, {
    onClick: () => console.log("clicked")
}, "Click me");
```

### Creating Modals

```js
Spicetify.PopupModal.display({
    title: "My Modal",
    content: React.createElement("div", null, "Hello!"),
});
```

---

## Spicetify Creator

For a better development experience with TypeScript, JSX, and hot reloading, use **[Spicetify Creator](/docs/development/spicetify-creator/the-basics)**.

```bash
npx spicetify-creator
```

Benefits:
- TypeScript support with full type definitions
- JSX syntax (no manual `React.createElement`)
- Hot reloading during development
- Build tooling (bundling, minification)

---

## Best Practices

### Performance

- Avoid polling. Use event listeners when possible
- Debounce expensive operations
- Clean up listeners when no longer needed

### Error Handling

```js
try {
    const data = await Spicetify.CosmosAsync.get("...");
} catch (error) {
    console.error("Request failed:", error);
    Spicetify.showNotification("Something went wrong", true);
}
```

### Compatibility

- Test with both light and dark themes
- Check for API existence before using (APIs can change between Spotify versions)
- Use feature detection:

```js
if (Spicetify.Topbar?.Button) {
    // Safe to use
}
```

### Naming

- Use a unique prefix for localStorage keys: `myExtension:key`
- Use descriptive function and variable names

---

## Publishing

To share your extension:

1. **GitHub**: Create a repository with your extension file(s) and installation instructions

2. **Marketplace**: Submit to the [Spicetify Marketplace](https://github.com/spicetify/marketplace) for easy discovery and installation

---

## Resources

- **[API Wrapper Reference](/docs/development/api-wrapper)**: Complete API documentation
- **[Spicetify Creator](/docs/development/spicetify-creator/the-basics)**: TypeScript development tool
- **[Built-in Extensions](https://github.com/spicetify/cli/tree/main/Extensions)**: Reference implementations
- **[Marketplace Extensions](https://github.com/spicetify/marketplace)**: Community examples


=== docs/development/index.md ===
---
title: Development
---

```mdx-code-block
import DocCardList from '@theme/DocCardList';
import {useCurrentSidebarCategory} from '@docusaurus/theme-common';

<DocCardList items={useCurrentSidebarCategory().items}/>
```


=== docs/development/js-modules.md ===
---
title: Javascript NPM Modules
description: 📦 Using NPM Modules for Spicetify.
---

Since v0.9.8, Spicetify injects extension with file extension `.mjs` as a script with type="module" and automatically symlink `node_modules` folder found in user's Extensions folder to `zlink` app.

In Javascript module, Javascript would work just the same as normal script but now you can use `import` to include other Javascript files. [Click here for details](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Modules).

Node Package Manager (NPM) is a commandline app bundled with NodeJS. You can use it to download and install hundreds of utility packages to ease your development process. Since Spicetify symlinks `node_modules` to Spotify main app folder, all packages files are mutually linked and available for you to use in your extension. Simply just use `import`:

```js
import './node_modules/package_name/file.js';
```

Careful! Javascript file you import has to be supported by Browser, not NodeJS. In other words, that Javascript should not use `require` function to include other packages or NodeJS API. Whenever you decide to use a package, check its readme page to see if its author already export distributed file as ES6 Module.

### Example

For Japanese studying purpose, I'm developing an extension that shows the Romaji form of Japanese track titles or artist names.

The idea is when I right click at track name and choose Show Romaji:

![img1](/images/development/js-modules-1.png)

Result should show as a notification:

![img2](/images/development/js-modules-2.png)

To translate Japanese text to Romaji, I use a package named [kuroshiro](https://github.com/hexenq/kuroshiro). Luckily, this package will export distribution files as ES6 Module. This is quite important because package itself relies on other utilities packages too. When it is compiled as an ES6 module, everything is transpiled to Browser supported Javascript and combined in one file. Moreover, kuroshiro also needs [kuroshiro-analyzer-kuromoji](https://github.com/hexenq/kuroshiro-analyzer-kuromoji) package to be usable, which relies on dictionaries gzip files. You can see there is no easy way to utiltise both packages and their external files if we use traditional Javascript extension.

**Following are steps to make and install this extension from scratch:**

1. First, change director to the user's `Extensions` folder:

- **Windows, in Powershell:**

```powershell
cd "$(dirname "$(spicetify -c)")/Extensions"
```

- **Linux and MacOS**, in Bash:

```
cd "$(dirname "$(spicetify -c)")/Extensions"
```

2. Now install needed packages with NPM

```bash
npm install kuroshiro kuroshiro-analyzer-kuromoji
```

Go to user's `Extensions` folder, you can see `node_modules` folder is created and contains all installed packages files. Next, go to the `kuroshiro` and `kuroshiro-analyzer-kuromoji`folder to locate ES6 Module distribution files.

3. After that I can comfortably include both of them in my extension. Following is extension code:

**romaji.mjs**:

```javascript
import './node_modules/kuroshiro/dist/kuroshiro.min.js';
import './node_modules/kuroshiro-analyzer-kuromoji/dist/kuroshiro-analyzer-kuromoji.min.js';

const kuroshiro = new Kuroshiro.default();
kuroshiro.init(new KuromojiAnalyzer());

function converter(input) {
  return kuroshiro.convert(input, {
    to: 'romaji',
    mode: 'spaced',
    romajiSystem: 'passport',
  });
}

const fetchAlbum = async (uri) => {
  const res = await Spicetify.CosmosAsync.get(`hm://album/v1/album-app/album/${uri.split(':')[2]}/desktop`);
  return res.name;
};

const fetchShow = async (uri) => {
  const res = await Spicetify.CosmosAsync.get(
    `sp://core-show/v1/shows/${uri.split(':')[2]}?responseFormat=protobufJson`,
    {
      policy: { list: { index: true } },
    }
  );
  return res.header.showMetadata.name;
};

const fetchArtist = async (uri) => {
  const res = await Spicetify.CosmosAsync.get(`hm://artist/v1/${uri.split(':')[2]}/desktop?format=json`);
  return res.info.name;
};

const fetchTrack = async (uri) => {
  const res = await Spicetify.CosmosAsync.get(`https://api.spotify.com/v1/tracks/${uri.split(':')[2]}`);
  return res.name;
};

const fetchEpisode = async (uri) => {
  const res = await Spicetify.CosmosAsync.get(`https://api.spotify.com/v1/episodes/${uri.split(':')[2]}`);
  return res.name;
};

const fetchPlaylist = async (uri) => {
  const res = await Spicetify.CosmosAsync.get(`sp://core-playlist/v1/playlist/${uri}/metadata`, {
    policy: { name: true },
  });
  return res.metadata.name;
};

async function showRomaji([uri]) {
  const type = uri.split(':')[1];
  let name;
  switch (type) {
    case Spicetify.URI.Type.TRACK:
      name = await fetchTrack(uri);
      break;
    case Spicetify.URI.Type.ALBUM:
      name = await fetchAlbum(uri);
      break;
    case Spicetify.URI.Type.ARTIST:
      name = await fetchArtist(uri);
      break;
    case Spicetify.URI.Type.SHOW:
      name = await fetchShow(uri);
      break;
    case Spicetify.URI.Type.EPISODE:
      name = await fetchEpisode(uri);
      break;
    case Spicetify.URI.Type.PLAYLIST:
    case Spicetify.URI.Type.PLAYLIST_V2:
      name = await fetchPlaylist(uri);
      break;
  }
  if (Kuroshiro.default.Util.hasJapanese(name)) {
    name = await converter(name);
    name = name.replace(/(^|\s)\S/g, (t) => t.toUpperCase());
  }
  Spicetify.showNotification(name);
}

new Spicetify.ContextMenu.Item(`Show Romaji`, showRomaji).register();
```

Save this file to `Extensions` folder.

4. Finally, push my extension to Spotify:

```bash
spicetify config extensions romaji.mjs
spicetify apply
```


=== docs/development/react-devtools.md ===
---
title: React Developer Tools
description: Installing and using React Developer Tools in Spotify.
---

To install React Developer Tools in Spotify:

1. Launch Spotify in the developer mode (and with the `--enable-chrome-runtime` flag if you are using **Spotify versions older than 1.2.34**; without it, React Developer Tools won't work)
2. Press *Ctrl + Shift + T*
3. Press *Ctrl + N*
4. Navigate to the React Developer Tools page on Chrome Web Store using the address bar: `https://chromewebstore.google.com/detail/react-developer-tools/fmkadmapgofadopljbjfkapdkoienihi`
5. Press the "Add to Chrome" button
6. Confirm your choice

You may need to press *F5* to get the extension working.

If you get an error saying that you're not allowed to install extensions, try using this switch: `--allowlisted-extension-id=fmkadmapgofadopljbjfkapdkoienihi`.

You can find more info about React Developer Tools on the [official React website](https://react.dev/learn/react-developer-tools).


=== docs/development/spicetify-creator/building-and-testing.md ===
---
title: Building And Testing
description: 🛠 Ensuring the quality of your creations.
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

## Building

Open a terminal in the project's directory and run this:

<Tabs groupId="package-managers">
  <TabItem value="npm" label="npm" default>
    ```shell
    npm run build
    ```
  </TabItem>
  <TabItem value="yarn" label="Yarn">
    ```shell
    yarn run build
    ```
  </TabItem>
  <TabItem value="pnpm" label="pnpm">
    ```shell
    pnpm build
    ```
  </TabItem>
</Tabs>

Then make sure you've added your app to Spicetify's config by running this:

<Tabs groupId="app-types">
  <TabItem value="extension" label="Extension" default>
    ```shell
    spicetify config extensions my-app.js
    ```
  </TabItem>
  <TabItem value="custom-app" label="Custom App">
    ```shell
    spicetify config custom_apps my-app
    ```
  </TabItem>
</Tabs>

Finally, do

```shell
spicetify apply
```

and you'll see your app in Spotify.

## Watching

Please first [build your app](#building) at least once before watching.

Watching means that it'll rebuild the app every time the code changes.  
Go into your project's directory and enter the following command:
<Tabs groupId="package-managers">
<TabItem value="npm" label="npm" default>
    ```shell
    npm run watch
    ```
  </TabItem>
  <TabItem value="yarn" label="Yarn">
    ```shell
    yarn run watch
    ```
  </TabItem>
<TabItem value="pnpm" label="pnpm" default>
    ```shell
    pnpm watch
    ```
  </TabItem>
</Tabs>

Then, run Spotify in watch mode:

<Tabs groupId="app-types">
  <TabItem value="extension" label="Extension" default>
    ```shell
    spicetify watch -le
    ```
  </TabItem>
  <TabItem value="custom-app" label="Custom App">
    ```shell
    spicetify watch -la
    ```
  </TabItem>
</Tabs>

## Building locally

If you want to upload the build files with your repository or just see them, you can do:

<Tabs groupId="package-managers">
  <TabItem value="npm" label="npm" default>
    ```shell
    npm run build-local
    ```
  </TabItem>
  <TabItem value="yarn" label="Yarn">
    ```shell
    yarn run build-local
    ```
  </TabItem>
  <TabItem value="pnpm" label="pnpm" default>
    ```shell
    pnpm build-local
    ```
  </TabItem>
</Tabs>

And the compiled files will be created in a local `dist` folder.


=== docs/development/spicetify-creator/create-custom-apps.md ===
---
title: Create Custom Apps
description: 🔧 Creating single-page apps for Spicetify.
---

Notes:

- It is recommended to learn React before starting to create Custom Apps.
- This tutorial assumes you have chosen to generate an example using Create Spicetify App.

After creating a new Spicetify Creator project and choosing "Custom App" as your app's type, your project's structure should look like this (With the generated example):

```
my-app/
  .gitattributes
  .gitignore
  package.json
  README.md
  tsconfig.json
  yarn.lock
  src/
    ...
  node_modules/
    ...
```

For now, we only care about the `src/` folder, whose structure looks like this

```
src/
  app.tsx
  settings.json
  extensions/
    extension.tsx
  css/
    icon.svg
    app.module.scss
  types/
    ...
```

`app.tsx` exports a React Component that will be mounted to Spotify every time the user enters your custom app.
It comes with an example of a simple counter using React's logic, and usages for SCSS modules.  
`settings.json` is a simple JSON file containing 4 keys:

```json
{
  "displayName": "My App", // The name of your app in the left sidebar
  "nameId": "my-app", // The id of your app
  "icon": "css/icon.svg", // The icon that will be displayed in the sidebar
  "activeIcon": "css/icon.svg" // The icon that will be displayed upon selecting the app in the sidebar.
}
```

The `extensions/` folder takes all the files inside it and transforms them into extensions that will run on Spotify's startup.  
The `extensions/extension.tsx` file is an example that says "Welcome!" whenever Spotify starts.

<details>
<summary>extension.tsx's content</summary>

```ts
(async () => {
  while (!Spicetify?.showNotification) {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  // Show message on start.
  Spicetify.showNotification('Welcome!');
})();
```

</details>


=== docs/development/spicetify-creator/create-extensions.md ===
---
title: Create Extensions
description: 🔨 Creating small addons for Spicetify.
---

Notes:

- This tutorial assumes you have chosen to generate an example using Create Spicetify App.

After creating a new Spicetify Creator project and choosing "Extension" as your app's type, your project's structure should look like this (With the generated example):

```
my-app/
  .gitattributes
  .gitignore
  package.json
  README.md
  tsconfig.json
  yarn.lock
  src/
    ...
  node_modules/
    ...
```

For now, we only care about the `src/` folder, whose structure looks like this

```
src/
  app.tsx
  settings.json
  types/
    ...
```

`app.tsx` exports a function that will be executed every time Spotify starts up.
It comes with an example that says "Hello!" to the user when Spotify starts up.  
`settings.json` is a simple JSON file containing 1 key:

```json
{
  "nameId": "my-app" // The id of your app
}
```


=== docs/development/spicetify-creator/the-basics.md ===
---
title: The Basics
description: 🤠 Spicetify Creator 101.
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

## Introduction

[Spicetify Creator](https://github.com/FlafyDev/spicetify-creator) is a tool to compile modern TypeScript/JavaScript code to Spicetify extensions and custom apps.

Its built-in features include:

- TypeScript and React syntax
- [Import node packages](#node-packages)
- [CSS/SCSS with PostCSS support](#css)
- Extremely fast compile time with esbuild.
- [Plugins](#plugins)

## Getting Started

The easiest way to start using Spicetify Creator is with Create Spicetify App.  
Create Spicetify App allows you to effortlessly create new Spicetify Creator projects through the terminal.

Install Node.js and either npm or Yarn.  
Open the terminal in your desired directory and enter the following command

<Tabs groupId="package-managers">
  <TabItem value="npm" label="npm" default>
    ```shell
      npx create-spicetify-app
    ```
  </TabItem>
  <TabItem value="yarn" label="Yarn">
    ```shell
      yarn create spicetify-app
    ```
  </TabItem>
  <TabItem value="pnpm" label="pnpm">
    ```shell
      pnpm create spicetify-app
    ```
  </TabItem>
</Tabs>

The command will ask you 3-4 simple questions about the app you plan to create and generate a Spicetify Creator project accordingly.  
After creation, read one of the following pages depending on what type of app you chose to create.

<div style={{width: '100%', justifyContent: 'center', alignItems: 'center', display: 'flex', padding: '20px', textAlign: 'center'}}>
  <span style={{margin: '0 20px'}}><a href='/docs/development/spicetify-creator/create-extensions'>Create Extensions</a></span>
  <span style={{margin: '0 20px'}}><a href='/docs/development/spicetify-creator/create-custom-apps'>Create Custom Apps</a></span>
</div>

## CSS

To apply a CSS/SCSS file to your app you have to import it like this:

```ts
import './my-css-file.css'; // For CSS
import './my-scss-file.scss'; // For SCSS
```

There is also support for [CSS Modules](https://github.com/css-modules/css-modules) and you import them like this:

```ts
import styles from './item-list.module.css'; // For CSS
import styles from './item-list.module.scss'; // For SCSS
```

## Node packages

You can use node packages in your app by installing them with your package manager.

<Tabs groupId="package-managers">
  <TabItem value="npm" label="npm" default>
    ```shell
      npm install <package-name>
    ```
  </TabItem>
  <TabItem value="yarn" label="Yarn">
    ```shell
      yarn add <package-name>
    ```
  </TabItem>
  <TabItem value="pnpm" label="pnpm">
    ```shell
      pnpm add <package-name>
    ```
  </TabItem>
</Tabs>

then simply import the package in the code and you're good to go.

```ts
import packageName from '<package-name>';
```

## Plugins

Plugins are node packages designed for Spicetify Creator projects, and they support either extensions, custom apps, or both.  
The convention is to name every plugin like so: `spcr-<name>`.

For a list of plugins: https://github.com/FlafyDev/spicetify-creator-plugins

To install and import a plugin:
<Tabs groupId="package-managers">
<TabItem value="npm" label="npm" default>
    ```shell
      npm install spcr-<name>
    ```
  </TabItem>
  <TabItem value="yarn" label="Yarn">
    ```shell
      yarn add spcr-<name>
    ```
  </TabItem>
  <TabItem value="pnpm" label="pnpm">
    ```shell
      pnpm add spcr-<name>
    ```
  </TabItem>
</Tabs>

```ts
import plugin from 'spcr-<name>';
```

#### Example of 2 plugins you can already use in your own apps:

- [spcr-settings](https://github.com/FlafyDev/spicetify-creator-plugins/tree/main/packages/spcr-settings)
- [spcr-navigation-bar](https://github.com/FlafyDev/spicetify-creator-plugins/tree/main/packages/spcr-navigation-bar)

## Update Spicetify Creator

<Tabs groupId="package-managers">
  <TabItem value="npm" label="npm" default>
    ```shell
      npm update spicetify-creator
    ```
  </TabItem>
  <TabItem value="yarn" label="Yarn">
    ```shell
      yarn upgrade spicetify-creator
    ```
  </TabItem>
  <TabItem value="pnpm" label="pnpm">
    ```shell
      pnpm update spicetify-creator
    ```
  </TabItem>
</Tabs>


=== docs/development/spotify-cli-flags.md ===
---
title: Spotify CLI Flags
description: 🚩 Flags to alter the behavior of the Spotify.
---

## List of flags

**When adding flags to your spicetify config, separate each one with `|`.**

| Flag                                   | Description                                                                                                                                                                                          |
| -------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--allow-upgrades`                     |                                                                                                                                                                                                      |
| `--append-log-file`                    |                                                                                                                                                                                                      |
| `--app-directory=<path>`               | Specify the "Apps" directory path. Used by spicetify for modifying Spotify from Microsoft Store.                                                                                                      |
| `--app-icon-overlay`                   |                                                                                                                                                                                                      |
| `--apr`                                |                                                                                                                                                                                                      |
| `--audio-api`                          |                                                                                                                                                                                                      |
| `--autostart`                          |                                                                                                                                                                                                      |
| `--bridge-log-filename`                |                                                                                                                                                                                                      |
| `--cache-path=<path>`                  | Use as root for the cache directory.                                                                                                                                                                 |
| `--campaign-id`                        |                                                                                                                                                                                                      |
| `--connect-debug-level`                |                                                                                                                                                                                                      |
| `--disable-cef-views`                  |                                                                                                                                                                                                      |
| `--disable-crash-reporting`            |                                                                                                                                                                                                      |
| `--disable-update-restarts`            |                                                                                                                                                                                                      |
| `--disallow-multiple-instances`        |                                                                                                                                                                                                      |
| `--enable-audio-graph`                 |                                                                                                                                                                                                      |
| `--enable-cef-views`                   |                                                                                                                                                                                                      |
| `--enable-chrome-runtime`              | Switches runtime from Alloy to Chrome on Spotify versions below 1.2.34. See [this issue on GitHub](https://github.com/chromiumembedded/cef/issues/3685) for details about the differences.           |
| `--enable-developer-mode`              | Used to enable the developer mode like `spicetify enable-devtools`. Stopped working long time ago.                                                                                                   |
| `--event-sender-send-interval`         |                                                                                                                                                                                                      |
| `--experimental-languages`             |                                                                                                                                                                                                      |
| `--experimental-network`               |                                                                                                                                                                                                      |
| `--force-auto-update`                  |                                                                                                                                                                                                      |
| `--force-cef-http`                     |                                                                                                                                                                                                      |
| `--immediate-widevine-cdm-download`    |                                                                                                                                                                                                      |
| `--log-detailed-request-account`       |                                                                                                                                                                                                      |
| `--log-file=<path>`                    | Save log output to file (extension needs to be '.log').                                                                                                                                              |
| `--maximized`                          |                                                                                                                                                                                                      |
| `--minimum-update-request-interval`    |                                                                                                                                                                                                      |
| `--minimized`                          | Start the app with the window minimized. Only works on Windows.                                                                                                                                      |
| `--mu=<value>`                         | Start with a special cache directory. Allows you to run multiple clients at the same time. Value can be anything (will be used as part of the cache name).                                           |
| `--password=<password>`                | Use to automatically sign in on startup. Use together with `--username`. No longer works.                                                                                                            |
| `--performance-tracing`                |                                                                                                                                                                                                      |
| `--product-version`                    |                                                                                                                                                                                                      |
| `--protocol-uri=<uri>`                 | Identical to --uri, but only used from the Windows protocol handler, so we can apply extra security restrictions.                                                                                    |
| `--remote-allow-origins=<url>`         | Required to use remote debugging since Spotify 1.2.8 due to security changes in Chromium 111. Example configuration: `--remote-debugging-port=8088 \| --remote-allow-origins=http://localhost:8088`. |
| `--remote-app-config`                  |                                                                                                                                                                                                      |
| `--remote-debugging-port=<port>`       | Enable remote debugging. Use together with `--remote-allow-origins`.                                                                                                                                 |
| `--remember-cmd-login`                 |                                                                                                                                                                                                      |
| `--show-console`                       | Show more log output.                                                                                                                                                                                |
| `--startup-success-file-path`          |                                                                                                                                                                                                      |
| `--test-auto-update-success-file-path` |                                                                                                                                                                                                      |
| `--trace-file=<path>`                  | Save a trace file to this path.                                                                                                                                                                      |
| `--trigger-ta-crash`                   |                                                                                                                                                                                                      |
| `--update-endpoint-override=<url>`     | Can be used to disable Spotify updates. Example configuration: `--update-endpoint-override=http://localhost`.                                                                                        |
| `--update-immediately`                 |                                                                                                                                                                                                      |
| `--upgrade-failed`                     |                                                                                                                                                                                                      |
| `--uri=<uri>`                          | Start the client normally, but automatically navigate to the URI when initialized.                                                                                                                   |
| `--use-event-sender-test-transport`    |                                                                                                                                                                                                      |
| `--user-agent-product`                 |                                                                                                                                                                                                      |
| `--username=<username>`                | Use to automatically sign in on startup. Use together with `--password`. No longer works.                                                                                                            |
| `--weblogin-endpoint`                  |                                                                                                                                                                                                      |

**When added to spicetify config, the flags will only be applied when you launch Spotify using spicetify.** But you can also add them to the Spotify shortcut (on Windows) or `.desktop` file (on Linux) and use it to launch Spotify.

Most of the flags with some descriptions are taken directly from the Spotify executable.

## See also

- [General documentation on Chromium command-line switches](https://www.chromium.org/developers/how-tos/run-chromium-with-flags)

- [List of Chromium command-line switches](https://peter.sh/experiments/chromium-command-line-switches)

- List of CEF command-line switches in the source code: [1](https://github.com/chromiumembedded/cef/blob/master/tests/shared/common/client_switches.cc), [2](https://github.com/chromiumembedded/cef/blob/master/libcef/common/cef_switches.cc)

Don't expect every switch to work.

## Experimental features

Some Chromium experimental features can be enabled with `--enable-features=<comma-separated feature list>`, some require both a switch and a feature. Smooth scrolling is an example: `--enable-smooth-scrolling | --enable-features=WindowsScrollingPersonality`.
There is no list of experimental features and they vary from version to version.

To enable experimental features in **Spotify newer than 1.2.33**:

1. Launch it in the developer mode
2. Press *Ctrl + Shift + T*
3. Press *Ctrl + N*
4. Navigate to the `chrome://flags` page using the address bar
5. Enable the ones you want
6. Press the "Relaunch" button

If you are using **Spotify older than 1.2.34**:

1. Launch it with the `--enable-chrome-runtime` switch and developer mode enabled
2. Press *Ctrl + Shift + T*
3. Press *Ctrl + N*
4. Navigate to the `chrome://flags` page using the address bar
5. Enable the ones you want
6. Press the "Relaunch" button
7. Restart Spotify with the `--enable-chrome-runtime` switch and developer mode enabled
8. Press *Ctrl + Shift + T*
9. Click the `chrome://version` link
10. Copy and paste the flags between `--flag-switches-begin` `--flag-switches-end` into your spicetify config and/or the shortcut/`.desktop` file you use to launch Spotify

The `--enable-chrome-runtime` switch and developer mode are not required for experimental features to work.


=== docs/development/themes.md ===
---
title: Themes
description: ✨ Creating Themes for Spicetify.
---

## Theme Structure

```text
~/.config/spicetify/Themes/       # Linux/macOS
%appdata%\spicetify\Themes\       # Windows
└── MyTheme/
    ├── color.ini                 # Required: color definitions
    ├── user.css                  # Required: custom CSS rules
    ├── theme.js                  # Optional: theme-specific JavaScript
    └── assets/                   # Optional: images, fonts, etc.
        └── ...
```

## Theme Location

There are 2 places you can put your themes:

1. `Themes` folder in Home directory

| Platform            | Path                              |
| ------------------- | --------------------------------- |
| **Windows**         | `%appdata%\spicetify\Themes`      |
| **Linux**/**MacOS** | `~/.config/spicetify/Themes`      |

2. `Themes` folder in Spicetify executable directory

If there are 2 themes with the same name, the theme in the Home directory is prioritized.

Every theme should contain:

- `color.ini`: stores colors values that later will be converted to CSS variables
- `user.css`: set of custom CSS rules to manipulate, hide, move UI elements.

Color value can be in several formats and forms:

- Hex: e.g `#FF0000`, `#1258F6`, `#F55`
- Decimal: e.g `255,255,255`, `50,80,120`
- Environment variables can be used in place of color.

  - Syntax: `${<variable name>}`
  - Example usage: `text = ${LIGHT_GREY}`

- **[Linux]** You can use XResources variable in place of color. Extremely useful for who uses `pywal` to generate color scheme.
  - Syntax: `${xrdb:<variable name>}` or `${xrdb:<variable name>:<fallback value>}`
  - Example usage:

```
[Base]
text     = ${xrdb:color14}
subtext  = ${xrdb:foreground:#FFF}
player   = ${xrdb:background}
...
```



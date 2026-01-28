// Category types
export type Category = "text" | "image" | "numeric" | "secure";

// Clipboard data types
export interface TextData {
  type: "text";
  text: string;
  preview: string;
}

export interface ImageData {
  type: "image";
  base64: string;
  width: number;
  height: number;
  format: string;
}

export type ClipboardData = TextData | ImageData;

// Clipboard content
export interface ClipboardContent {
  id: string;
  category: Category;
  data: ClipboardData;
  copiedAt: string;
  source?: string;
}

// Position and Size
export interface Position {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

// Whiteboard item
export interface WhiteboardItem {
  id: string;
  content: ClipboardContent;
  position: Position;
  size: Size;
  parentGroup: string | null;
  shortcut: string | null;
  createdAt: string;
  updatedAt: string;
}

// Group
export interface Group {
  id: string;
  name: string;
  position: Position;
  collapsed: boolean;
  parentGroup: string | null;
  children: string[];
  shortcut: string | null;
  color: string | null; // Hex color for group
  createdAt: string;
  updatedAt: string;
}

// Whiteboard state
export interface WhiteboardState {
  items: Record<string, WhiteboardItem>;
  groups: Record<string, Group>;
  rootItems: string[];
}

// View mode
export type ViewMode = "list" | "whiteboard";

// Context menu types
export interface ContextMenuState {
  show: boolean;
  x: number;
  y: number;
  target: ContextMenuTarget | null;
}

export type ContextMenuTarget =
  | { type: "item"; id: string }
  | { type: "group"; id: string }
  | { type: "whiteboard"; position: Position };

// Shortcut match result
export interface ShortcutMatch {
  type: "item" | "group";
  id: string;
  shortcut: string;
  name: string;
  parentGroup: string | null;
}

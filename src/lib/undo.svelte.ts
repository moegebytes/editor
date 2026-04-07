import { isWordBoundary } from './utils';

const MAX_UNDO = 200;

export type Command =
  | { kind: 'editText'; index: number; oldText: string | null; newText: string | null }
  | { kind: 'editNotes'; index: number; oldNotes: string[]; newNotes: string[] }
  | { kind: 'confirm'; index: number }
  | { kind: 'unconfirm'; index: number };

export type UndoEntry = Command | Command[];

function isEditText(entry: UndoEntry): entry is Command & { kind: 'editText' } {
  return !Array.isArray(entry) && entry.kind === 'editText';
}

export class UndoStack {
  #undo: UndoEntry[] = [];
  #redo: UndoEntry[] = [];

  canUndo = $state(false);
  canRedo = $state(false);

  #sync() {
    this.canUndo = this.#undo.length > 0;
    this.canRedo = this.#redo.length > 0;
  }

  push(entry: UndoEntry) {
    this.#undo.push(entry);
    if (this.#undo.length > MAX_UNDO) this.#undo.splice(0, this.#undo.length - MAX_UNDO);
    this.#redo.length = 0;
    this.#sync();
  }

  coalesceText(index: number, oldText: string | null, newText: string | null): boolean {
    const last = this.#undo.at(-1);
    if (last && isEditText(last) && last.index === index && !isWordBoundary(last.newText, newText)) {
      last.newText = newText;
      this.#redo.length = 0;
      this.#sync();
      return true;
    }
    this.push({ kind: 'editText', index, oldText, newText });
    return false;
  }

  groupWithLast(command: Command) {
    const last = this.#undo.at(-1);
    if (last && isEditText(last) && 'index' in command && last.index === command.index) {
      this.#undo.pop();
      this.#undo.push([last, command]);
    } else {
      this.#undo.push(command);
    }
    this.#redo.length = 0;
    this.#sync();
  }

  popUndo(): UndoEntry | undefined {
    const entry = this.#undo.pop();
    if (entry) this.#redo.push(entry);
    this.#sync();
    return entry;
  }

  popRedo(): UndoEntry | undefined {
    const entry = this.#redo.pop();
    if (entry) this.#undo.push(entry);
    this.#sync();
    return entry;
  }

  clear() {
    this.#undo.length = 0;
    this.#redo.length = 0;
    this.#sync();
  }
}

import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Brush,
  Check,
  Droplet,
  Eraser,
  FilePlus2,
  FolderOpen,
  Highlighter,
  LogOut,
  PaintBucket,
  Pencil,
  Redo2,
  Save,
  Undo2,
  type LucideIcon,
} from "lucide-react";
import { DrawingPad, type Brush as InkBrush } from "./pad";
import { Gallery, type GalleryView } from "./Gallery";
import "./style.css";

const colors = [
  ["Red", "#ff3b30"],
  ["Coral", "#ff6a2a"],
  ["Orange", "#ff9500"],
  ["Amber", "#ffb020"],
  ["Gold", "#ffcc00"],
  ["Yellow", "#e5e500"],
  ["Lime", "#9bd400"],
  ["Green", "#34c759"],
  ["Mint", "#00c7be"],
  ["Teal", "#00a7a0"],
  ["Sky blue", "#32ade6"],
  ["Blue", "#007aff"],
  ["Indigo", "#3b4bff"],
  ["Violet", "#6c5ce7"],
  ["Purple", "#af52de"],
  ["Orchid", "#e056fd"],
  ["Rose", "#ff2d55"],
  ["Pink", "#ff5e9a"],
  ["Tan", "#a2845e"],
  ["Brown", "#7f5a3a"],
  ["Black", "#1c1c1e"],
  ["Gray", "#4e4e52"],
  ["Silver", "#cfcfd4"],
  ["White", "#ffffff"],
];
const sizes = [
  2, 3, 4, 5, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32, 36, 40, 48, 56, 64,
];
const styles = [
  { name: "Pencil", icon: Pencil, multiplier: 1, opacity: 255 },
  { name: "Marker", icon: Highlighter, multiplier: 1.3, opacity: 170 },
  { name: "Watercolor", icon: Droplet, multiplier: 1.8, opacity: 105 },
];
type Tool = "Brush" | "Eraser" | "Fill";
type Modal =
  | { kind: "Gallery"; view: GalleryView }
  | { kind: "Open"; id: string }
  | { kind: "New" | "Quit" };
const prompts = {
  New: "Start a new drawing?",
  Open: "Replace your drawing?",
  Quit: "Leave Alidraw?",
};

function Button({
  name,
  icon: Icon,
  active,
  disabled,
  onClick,
}: {
  name: string;
  icon: LucideIcon;
  active?: boolean;
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={name}
      title={name}
      aria-pressed={active}
      disabled={disabled}
      onClick={onClick}
    >
      <Icon aria-hidden="true" />
      <span>{name}</span>
    </button>
  );
}

function App() {
  const [tool, setTool] = useState<Tool>("Brush");
  const [style, setStyle] = useState(0);
  const [size, setSize] = useState(5);
  const [color, setColor] = useState(20);
  const [history, setHistory] = useState({ undo: false, redo: false });
  const [notice, setNotice] = useState<{ message: string }>();
  const setStatus = useCallback(
    (message: string) => setNotice({ message }),
    [],
  );
  const [modal, setModal] = useState<Modal>();
  const [preview, setPreview] = useState(false);
  const [busy, setBusy] = useState(false);
  const locked = useRef(false);
  const area = useRef<HTMLElement>(null);
  const dialog = useRef<HTMLDialogElement>(null);
  const pad = useRef<DrawingPad | null>(null);
  const brush = useRef<InkBrush>({
    color: colors[20][1],
    width: 8,
    opacity: 255,
    fill: false,
  });

  useLayoutEffect(() => {
    const spec = styles[tool === "Eraser" ? 0 : style];
    brush.current = {
      color: tool === "Eraser" ? "#ffffff" : colors[color][1],
      width: sizes[size] * spec.multiplier,
      opacity: spec.opacity,
      fill: tool === "Fill",
    };
  }, [tool, style, size, color]);

  const lock = (value: boolean) => {
    locked.current = value;
    pad.current?.setBlocked(value);
    setBusy(value);
  };
  const ask = (kind: "New" | "Quit") => {
    lock(true);
    setModal({ kind });
  };
  const cancel = () => {
    setModal(undefined);
    lock(false);
  };

  useEffect(() => {
    if (modal) dialog.current?.showModal();
    else dialog.current?.close();
  }, [modal]);

  useEffect(() => {
    if (!notice) return;
    const timeout = setTimeout(() => setNotice(undefined), 4500);
    return () => clearTimeout(timeout);
  }, [notice]);

  useEffect(() => {
    if (!area.current) return;
    const drawing = new DrawingPad(
      area.current,
      () => brush.current,
      setHistory,
      setStatus,
      setPreview,
    );
    pad.current = drawing;
    const win = getCurrentWindow();
    const unlisten = win
      .onCloseRequested((e) => {
        e.preventDefault();
        if (!locked.current) ask("Quit");
      })
      .catch((err) => {
        setStatus(String(err));
        return () => {};
      });
    const key = (e: KeyboardEvent) => {
      if (e.repeat || locked.current) return;
      if (["F8", "F9", "F11"].includes(e.key)) e.preventDefault();
      if (e.key === "F8") drawing.toggleDiagnostics();
      if (e.key === "F9") {
        setTool("Brush");
        drawing.toggleLocalOnly();
      }
      if (e.key === "F11")
        void win
          .isFullscreen()
          .then((v) => win.setFullscreen(!v))
          .catch((err) => setStatus(String(err)));
    };
    window.addEventListener("keydown", key);
    return () => {
      void unlisten.then((fn) => fn());
      window.removeEventListener("keydown", key);
      drawing.dispose();
    };
  }, []);

  async function file(action: "Open" | "Save") {
    const drawing = pad.current;
    if (!drawing || locked.current) return;
    lock(true);
    try {
      await drawing.idle();
      if (action === "Open") {
        const view = await invoke<GalleryView>("list_drawings");
        setModal({ kind: "Gallery", view });
        return;
      }
      await invoke("save_image");
      setStatus("Saved!");
    } catch (err) {
      console.error(err);
      setStatus(
        action === "Save"
          ? "Couldn't save your drawing. Try again."
          : "Couldn't open My drawings. Try again.",
      );
    }
    lock(false);
  }

  async function confirm(choice: Exclude<Modal, { kind: "Gallery" }>) {
    const drawing = pad.current;
    if (!drawing) return;
    setModal(undefined);
    try {
      await drawing.idle();
      if (choice.kind === "Quit") await getCurrentWindow().destroy();
      else if (choice.kind === "Open") {
        await drawing.edit(
          "open_image",
          { id: choice.id, ...drawing.size() },
          "Couldn't open that picture.",
        );
      } else await drawing.edit("new_paper", drawing.size());
    } catch (err) {
      setStatus(String(err));
    } finally {
      lock(false);
    }
  }

  return (
    <main>
      <section ref={area} className="drawing" aria-label="Drawing paper" />
      <aside aria-label="Drawing tools" inert={busy}>
        <header>
          <span className="brand-mark" aria-hidden="true">
            <Pencil />
          </span>
          <h1>AliDraw</h1>
        </header>
        <div className="choices">
          <fieldset>
            <legend>Tools</legend>
            <div className="three tools">
              <Button
                name="Brush"
                icon={Brush}
                active={tool === "Brush"}
                onClick={() => setTool("Brush")}
              />
              <Button
                name="Eraser"
                icon={Eraser}
                active={tool === "Eraser"}
                onClick={() => setTool("Eraser")}
              />
              <Button
                name="Fill"
                disabled={preview}
                icon={PaintBucket}
                active={tool === "Fill"}
                onClick={() => setTool("Fill")}
              />
            </div>
          </fieldset>
          <fieldset disabled={tool !== "Brush"}>
            <legend>Brush style</legend>
            <div className="three">
              {styles.map((s, i) => (
                <Button
                  key={s.name}
                  name={s.name}
                  icon={s.icon}
                  active={i === style}
                  onClick={() => setStyle(i)}
                />
              ))}
            </div>
          </fieldset>
          <div className="paint-controls">
            <fieldset disabled={tool === "Eraser"}>
              <legend>Colors</legend>
              <div className="swatches">
                {colors.map(([name, hex], i) => (
                  <button
                    key={name}
                    aria-label={name}
                    title={name}
                    aria-pressed={color === i}
                    style={{ backgroundColor: hex }}
                    onClick={() => setColor(i)}
                  >
                    {color === i && <Check aria-hidden="true" />}
                  </button>
                ))}
              </div>
            </fieldset>
            <fieldset className="size" disabled={tool === "Fill"}>
              <legend>Size</legend>
              <input
                aria-label="Brush size"
                type="range"
                min="0"
                max={sizes.length - 1}
                value={size}
                aria-valuetext={sizes[size] + " pixels"}
                onChange={(e) => setSize(Number(e.target.value))}
              />
              <span className="size-number">{sizes[size]}</span>
            </fieldset>
          </div>
        </div>
        <footer>
          <div className="two">
            <Button
              name="Undo"
              icon={Undo2}
              disabled={preview || !history.undo}
              onClick={() => void pad.current?.edit("history", { redo: false })}
            />
            <Button
              name="Redo"
              icon={Redo2}
              disabled={preview || !history.redo}
              onClick={() => void pad.current?.edit("history", { redo: true })}
            />
          </div>
          <div className="four">
            <Button
              name="Save"
              disabled={preview}
              icon={Save}
              onClick={() => void file("Save")}
            />
            <Button
              name="Open"
              disabled={preview}
              icon={FolderOpen}
              onClick={() => void file("Open")}
            />
            <Button
              name="New"
              disabled={preview}
              icon={FilePlus2}
              onClick={() => ask("New")}
            />
            <Button name="Quit" icon={LogOut} onClick={() => ask("Quit")} />
          </div>
        </footer>
      </aside>
      {preview && (
        <div className="test-mode">
          Preview-only ink · F9 to return to your drawing
        </div>
      )}
      {notice && <output>{notice.message}</output>}
      <dialog
        ref={dialog}
        className={modal?.kind === "Gallery" ? "gallery" : "confirmation"}
        onCancel={(e) => {
          e.preventDefault();
          cancel();
        }}
        aria-labelledby="question"
      >
        {modal?.kind === "Gallery" ? (
          <Gallery
            entries={modal.view.entries}
            onBack={cancel}
            onOpen={(id) => {
              const choice = { kind: "Open", id } as const;
              if (modal.view.confirmOpen) setModal(choice);
              else void confirm(choice);
            }}
          />
        ) : (
          modal && (
            <>
              <h2 id="question">{prompts[modal.kind]}</h2>
              <p>Your current drawing will be closed.</p>
              <div>
                <button autoFocus onClick={cancel}>
                  Keep drawing
                </button>
                <button onClick={() => void confirm(modal)}>
                  {modal.kind}
                </button>
              </div>
            </>
          )
        )}
      </dialog>
    </main>
  );
}

createRoot(document.getElementById("root")!).render(<App />);

import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, ImageOff, Images } from "lucide-react";

export type GalleryView = { entries: string[]; confirmOpen: boolean };

export function Gallery({
  entries,
  onOpen,
  onBack,
}: {
  entries: string[];
  onOpen: (id: string) => void;
  onBack: () => void;
}) {
  const grid = useRef<HTMLDivElement>(null);
  // null marks an unreadable image. The other tiles stay available.
  const [previews, setPreviews] = useState<Map<string, string | null>>(
    () => new Map(),
  );

  useEffect(() => {
    if (!grid.current) return;
    const pending = new Set<Element>();
    const urls: string[] = [];
    let closed = false;
    let loading = false;

    // Decode one visible picture at a time so opening a large gallery is inexpensive.
    async function loadVisible() {
      if (loading) return;
      loading = true;
      for (const tile of pending) {
        pending.delete(tile);
        observer.unobserve(tile);
        const id = (tile as HTMLButtonElement).value;
        let url = null;
        try {
          const png = await invoke<ArrayBuffer>("drawing_thumbnail", { id });
          if (closed) return;
          url = URL.createObjectURL(new Blob([png], { type: "image/png" }));
          urls.push(url);
        } catch (error) {
          console.warn("Could not load drawing preview", error);
        }
        if (closed) return;
        setPreviews((current) => new Map(current).set(id, url));
      }
      loading = false;
    }

    const observer = new IntersectionObserver(
      (changes) => {
        for (const change of changes) {
          if (change.isIntersecting) pending.add(change.target);
          else pending.delete(change.target);
        }
        void loadVisible();
      },
      { root: grid.current },
    );
    for (const tile of grid.current.children) observer.observe(tile);
    return () => {
      closed = true;
      observer.disconnect();
      pending.clear();
      for (const url of urls) URL.revokeObjectURL(url);
    };
  }, [entries]);

  return (
    <>
      <div className="gallery-heading">
        <h2 id="question">
          <Images aria-hidden="true" />
          My drawings
        </h2>
        <button
          autoFocus
          type="button"
          className="gallery-back"
          onClick={onBack}
        >
          <ArrowLeft aria-hidden="true" /> Back to drawing
        </button>
      </div>
      {entries.length === 0 ? (
        <p className="gallery-empty">Save a drawing and it will appear here.</p>
      ) : (
        <div className="gallery-grid" ref={grid}>
          {entries.map((id, i) => {
            const preview = previews.get(id);
            return (
              <button
                key={id}
                value={id}
                type="button"
                className="gallery-tile"
                disabled={preview === null}
                aria-label={
                  preview === null
                    ? "Picture unavailable"
                    : `Open drawing ${i + 1}`
                }
                onClick={() => onOpen(id)}
              >
                {preview ? (
                  <img src={preview} alt="" />
                ) : preview === null ? (
                  <>
                    <ImageOff aria-hidden="true" />
                    Picture unavailable
                  </>
                ) : (
                  <Images aria-hidden="true" />
                )}
              </button>
            );
          })}
        </div>
      )}
    </>
  );
}

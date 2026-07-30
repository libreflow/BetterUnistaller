import { useEffect, useState } from "react";
import { listPrograms } from "./lib/api";
import type { Program } from "./lib/types";
import "./App.css";

export default function App() {
  const [programs, setPrograms] = useState<Program[]>([]);
  useEffect(() => {
    listPrograms().then(setPrograms);
  }, []);
  return (
    <main className="p-4">
      <p>{programs.filter((p) => !p.isSystemEntry).length} programmes</p>
    </main>
  );
}

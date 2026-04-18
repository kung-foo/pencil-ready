import { Palette } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { THEMES } from "@/lib/themes";

export function ThemeSwitcher({
  value,
  onChange,
}: {
  value: string;
  onChange: (id: string) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <Palette className="size-4 text-muted-foreground" />
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger className="w-[180px] h-8 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {THEMES.map((t) => (
            <SelectItem key={t.id} value={t.id}>
              <div className="flex flex-col">
                <span>{t.name}</span>
                <span className="text-xs text-muted-foreground">{t.hint}</span>
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

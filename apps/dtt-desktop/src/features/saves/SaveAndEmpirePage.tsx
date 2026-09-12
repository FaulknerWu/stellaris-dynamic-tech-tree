import { open } from "@tauri-apps/plugin-dialog";
import {
  Calendar,
  Check,
  Clock,
  Cloud,
  FileCode,
  Globe,
  HardDrive,
  Lock,
  RefreshCw,
  Sparkles,
  Upload,
  User,
  XCircle,
} from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { ErrorAlert } from "@/components/ErrorAlert";
import { IdBadge } from "@/components/IdBadge";
import { Panel } from "@/components/Panel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { SessionAction, SessionState } from "@/app/session";
import type {
  PlayerCountryCandidateDto,
  SaveIndexDto,
  SnapshotDto,
} from "@/ipc/bindings";
import { formatDate, formatFileSize } from "@/lib/format";
import { cn } from "@/lib/utils";

interface SaveAndEmpirePageProps {
  state: SessionState;
  dispatch: React.Dispatch<SessionAction>;
  onScanLibrary: (source?: "local" | "steam_cloud") => void;
  onSelectSave: (save: SaveIndexDto) => void;
  onInspectWithCountry: (countryId: number) => void;
}

const DEFAULT_VISIBLE_ID_COUNT = 12;

interface ExpandableIdListProps {
  items: string[];
  emptyText: string;
  showMoreLabel: (count: number) => string;
  showLessLabel: string;
}

function ExpandableIdList({
  items,
  emptyText,
  showMoreLabel,
  showLessLabel,
}: ExpandableIdListProps) {
  const [expanded, setExpanded] = useState(false);
  const visibleItems = expanded ? items : items.slice(0, DEFAULT_VISIBLE_ID_COUNT);
  const hiddenCount = items.length - visibleItems.length;

  if (items.length === 0) {
    return <span className="text-body text-muted-foreground">{emptyText}</span>;
  }

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-1.5">
        {visibleItems.map((item) => (
          <IdBadge key={item} value={item} />
        ))}
      </div>
      {items.length > DEFAULT_VISIBLE_ID_COUNT && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-7 px-2 text-meta"
          onClick={() => setExpanded((current) => !current)}
        >
          {expanded ? showLessLabel : showMoreLabel(hiddenCount)}
        </Button>
      )}
    </div>
  );
}

export function SaveAndEmpirePage({
  state,
  dispatch,
  onScanLibrary,
  onSelectSave,
  onInspectWithCountry,
}: SaveAndEmpirePageProps) {
  const { t } = useTranslation(["saves", "empire", "common"]);

  const [activeTab, setActiveTab] = useState("civics");

  const isScanning = state.libraryStatus === "scanning";
  const isInspecting = state.inspectionStatus === "inspecting";

  const allSavesWithCampaign = useMemo(() => {
    if (!state.library) return [];
    const list: {
      campaignKey: string;
      save: SaveIndexDto;
    }[] = [];

    if (state.manualSave) {
      list.push({
        campaignKey: state.manualSave.metadata?.name || t("saves:customSelection"),
        save: state.manualSave,
      });
    }

    for (const acc of state.library.accounts) {
      for (const camp of acc.campaigns) {
        for (const save of camp.saves) {
          list.push({
            campaignKey: camp.key,
            save,
          });
        }
      }
    }
    return list;
  }, [state.library, state.manualSave, t]);

  const availableSavesCount = allSavesWithCampaign.filter(
    (s) => s.save.state === "text",
  ).length;

  const handleManualOpenSave = async () => {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: t("saves:manualSelect"),
        filters: [{ name: "Stellaris Save", extensions: ["sav"] }],
      });

      if (typeof selected === "string") {
        const fileName = selected.split(/[\\/]/).pop() || "save.sav";
        const manualSaveItem: SaveIndexDto = {
          path: selected,
          fileName,
          modifiedAtMillis: Date.now(),
          fileSize: 0,
          state: "text",
        };
        dispatch({ type: "SET_MANUAL_SAVE", save: manualSaveItem });
        onSelectSave(manualSaveItem);
      }
    } catch (err) {
      console.error("Open manual save error:", err);
    }
  };

  const handleSourceChange = (value: unknown) => {
    const next = Array.isArray(value) ? value[0] : value;
    if (next === "local" || next === "steam_cloud") {
      dispatch({ type: "SET_SAVE_SOURCE", source: next });
      onScanLibrary(next);
    }
  };

  const handleSaveChange = (value: unknown) => {
    if (typeof value !== "string") return;
    const found = allSavesWithCampaign.find((item) => item.save.path === value);
    if (found && found.save.state === "text") {
      onSelectSave(found.save);
    }
  };

  const snapshot: SnapshotDto | null = state.inspection?.snapshot ?? null;
  const playerCandidates: PlayerCountryCandidateDto[] =
    state.inspection?.playerCountries ?? [];

  const emptyKey =
    state.saveSource === "steam_cloud" ? "empty.steamCloud" : "empty.local";

  return (
    <div className="grid h-full min-h-0 grid-cols-1 gap-4 lg:grid-cols-12">
      <Panel
        className="lg:col-span-5"
        title={t("saves:libraryTitle")}
        icon={Globe}
        description={t("saves:availableCount", { count: availableSavesCount })}
        action={
          <>
            <ToggleGroup
              value={[state.saveSource]}
              onValueChange={handleSourceChange}
              variant="outline"
              size="sm"
              spacing={0}
              disabled={isScanning}
              aria-label={t("saves:title")}
            >
              <ToggleGroupItem value="local" className="gap-1.5 px-2.5 text-body">
                <HardDrive className="size-3.5" />
                <span>{t("saves:sources.local")}</span>
              </ToggleGroupItem>
              <ToggleGroupItem
                value="steam_cloud"
                className="gap-1.5 px-2.5 text-body"
              >
                <Cloud className="size-3.5" />
                <span>{t("saves:sources.steamCloud")}</span>
              </ToggleGroupItem>
            </ToggleGroup>

            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleManualOpenSave}
              className="h-8 gap-1.5 border-dashed text-body"
            >
              <Upload className="size-3.5" />
              <span>{t("saves:manualSelectFile")}</span>
            </Button>

            <Button
              type="button"
              variant="ghost"
              size="icon"
              disabled={isScanning}
              onClick={() => onScanLibrary()}
              className="size-8"
              aria-label={t("common:actions.refresh")}
            >
              <RefreshCw
                className={`size-3.5 ${isScanning ? "animate-spin" : ""}`}
              />
            </Button>
          </>
        }
      >
        {state.libraryError && (
          <div className="mb-2.5">
            <ErrorAlert
              error={state.libraryError}
              onRemedy={() => onScanLibrary()}
              remedyText={t("common:actions.retry")}
            />
          </div>
        )}

        {isScanning ? (
          <div className="space-y-2">
            <Skeleton className="h-16 w-full rounded-lg" />
            <Skeleton className="h-16 w-full rounded-lg" />
            <Skeleton className="h-16 w-full rounded-lg" />
          </div>
        ) : allSavesWithCampaign.length === 0 ? (
          <Empty className="h-full min-h-0 border border-dashed border-border-subtle">
            <EmptyHeader>
              <EmptyTitle className="text-body">{t(`saves:${emptyKey}.title`)}</EmptyTitle>
              <EmptyDescription className="text-meta">
                {t(`saves:${emptyKey}.description`)}
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          <RadioGroup
            value={state.selectedSavePath ?? undefined}
            onValueChange={handleSaveChange}
            className="gap-1.5"
            aria-label={t("saves:listAriaLabel")}
          >
            {allSavesWithCampaign.map(({ campaignKey, save }) => {
              const isSelected = state.selectedSavePath === save.path;
              const isText = save.state === "text";
              const isBinary = save.state === "binary";
              const isCorrupt = save.state === "corrupt";

              const displayName =
                save.metadata?.name || campaignKey || save.fileName;
              const date = save.metadata?.date || "2200.01.01";
              const modified = formatDate(save.modifiedAtMillis);
              const size = formatFileSize(save.fileSize);

              return (
                <label
                  key={save.path}
                  className={cn(
                    "relative flex flex-col gap-1 rounded-lg border px-2.5 py-2 text-left transition-all select-none",
                    "has-[:focus-visible]:border-ring has-[:focus-visible]:ring-3 has-[:focus-visible]:ring-ring/50",
                    isText
                      ? "cursor-pointer hover:border-border hover:bg-muted"
                      : "cursor-not-allowed opacity-70",
                    isSelected && isText
                      ? "border-primary bg-muted font-medium"
                      : "border-border-subtle bg-card",
                  )}
                >
                  <RadioGroupItem
                    value={save.path}
                    disabled={!isText}
                    className="sr-only absolute size-px overflow-hidden"
                  />

                  <div className="flex items-center justify-between gap-2">
                    <div className="flex min-w-0 items-center gap-1.5">
                      <span
                        className={cn(
                          "size-2 shrink-0 rounded-full",
                          isText && "bg-success",
                          isBinary && "bg-muted-foreground",
                          isCorrupt && "bg-destructive",
                        )}
                      />
                      <span className="truncate text-body font-semibold text-foreground">
                        {displayName}
                      </span>
                    </div>

                    <div className="shrink-0">
                      {isText && (
                        <Badge
                          variant="success"
                          className="h-4 px-1.5 text-meta font-normal"
                        >
                          {t("saves:state.text.badge")}
                        </Badge>
                      )}
                      {isBinary && (
                        <Badge
                          variant="warning"
                          className="h-4 px-1.5 text-meta font-normal"
                        >
                          {t("saves:state.binary.badge")}
                        </Badge>
                      )}
                      {isCorrupt && (
                        <Badge
                          variant="destructive"
                          className="h-4 px-1.5 text-meta font-normal"
                        >
                          {t("saves:state.corrupt.badge")}
                        </Badge>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-3 font-mono text-meta text-muted-foreground">
                    <div className="flex items-center gap-1">
                      <Calendar className="size-3 text-primary" />
                      <span>{t("saves:gameDate", { date })}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Clock className="size-2.5 text-muted-foreground" />
                      <span>{modified}</span>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 truncate font-mono text-meta text-muted-foreground">
                    <FileCode className="size-3 shrink-0 text-muted-foreground" />
                    <span className="truncate">{save.fileName}</span>
                    <span>({size})</span>
                  </div>

                  {isBinary && (
                    <div className="mt-2 flex items-center gap-1.5 rounded border border-warning/30 bg-warning/10 px-2 py-1 text-meta text-warning">
                      <Lock className="size-3 shrink-0" />
                      <span>{t("saves:binaryWarning")}</span>
                    </div>
                  )}
                  {isCorrupt && (
                    <div className="mt-2 flex items-center gap-1.5 rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-meta text-destructive">
                      <XCircle className="size-3 shrink-0" />
                      <span>
                        {save.unavailableReason || t("saves:corruptFallback")}
                      </span>
                    </div>
                  )}
                </label>
              );
            })}
          </RadioGroup>
        )}
      </Panel>

      <Panel
        className="lg:col-span-7"
        title={t("saves:empireTitle")}
        icon={Sparkles}
      >
        {isInspecting ? (
          <div className="space-y-2 py-6">
            <div className="flex items-center justify-center gap-2 text-body text-muted-foreground">
              <Sparkles className="size-4 animate-spin text-primary" />
              <span>{t("empire:inspecting")}</span>
            </div>
            <Skeleton className="h-14 w-full rounded-md" />
            <Skeleton className="h-32 w-full rounded-md" />
          </div>
        ) : state.inspectionError ? (
          <ErrorAlert error={state.inspectionError} />
        ) : snapshot ? (
          <div className="space-y-3">
            {playerCandidates.length > 1 && (
              <div className="flex items-center justify-between pb-1">
                <span className="text-body font-medium text-muted-foreground">
                  {t("empire:playerCountrySwitch")}
                </span>
                <DropdownMenu>
                  <DropdownMenuTrigger
                    render={
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="h-7 max-w-44 shrink-0 gap-1.5 truncate text-body"
                      >
                        <User className="size-3" />
                        <span className="truncate">
                          {t("empire:countryLabel", {
                            id: state.selectedCountryId ?? playerCandidates[0]?.countryId,
                          })}
                        </span>
                      </Button>
                    }
                  />
                  <DropdownMenuContent align="end" className="w-56 text-body">
                    {playerCandidates.map((candidate) => (
                      <DropdownMenuItem
                        key={candidate.countryId}
                        onClick={() => {
                          dispatch({
                            type: "SELECT_COUNTRY_ID",
                            countryId: candidate.countryId,
                          });
                          onInspectWithCountry(candidate.countryId);
                        }}
                        className="justify-between"
                      >
                        <span>
                          {t("empire:countryLabel", { id: candidate.countryId })}
                        </span>
                        {state.selectedCountryId === candidate.countryId && (
                          <Check className="size-3 stroke-[3] text-primary" />
                        )}
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            )}

            <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
              <div className="rounded-lg bg-surface p-2 text-center">
                <div className="text-meta text-muted-foreground">
                  {t("empire:metrics.authority")}
                </div>
                <div className="mt-0.5 truncate text-body font-semibold text-foreground">
                  {snapshot.government.authority || t("empire:defaults.authority")}
                </div>
              </div>
              <div className="rounded-lg bg-surface p-2 text-center">
                <div className="text-meta text-muted-foreground">
                  {t("empire:metrics.origin")}
                </div>
                <div className="mt-0.5 truncate text-body font-semibold text-foreground">
                  {snapshot.government.origin || t("empire:defaults.origin")}
                </div>
              </div>
              <div className="rounded-lg bg-surface p-2 text-center">
                <div className="text-meta text-muted-foreground">
                  {t("empire:metrics.species")}
                </div>
                <div className="mt-0.5 truncate text-body font-semibold text-foreground">
                  {snapshot.founderSpecies.archetype || t("empire:defaults.species")}
                </div>
              </div>
              <div className="rounded-lg bg-surface p-2 text-center">
                <div className="text-meta text-muted-foreground">
                  {t("empire:metrics.countryType")}
                </div>
                <div className="mt-0.5 truncate text-body font-semibold text-foreground">
                  {snapshot.countryType || t("empire:defaults.countryType")}
                </div>
              </div>
            </div>

            <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
              <TabsList className="grid h-8 w-full grid-cols-3 p-0.5">
                <TabsTrigger value="civics" className="h-7 text-body">
                  {t("empire:tabs.civics")}
                </TabsTrigger>
                <TabsTrigger value="traits" className="h-7 text-body">
                  {t("empire:tabs.traits")}
                </TabsTrigger>
                <TabsTrigger value="ascension" className="h-7 text-body">
                  {t("empire:tabs.ascension")}
                </TabsTrigger>
              </TabsList>

              <TabsContent value="civics" className="space-y-2.5 pt-2">
                <div className="space-y-1">
                  <div className="text-meta font-medium text-muted-foreground">
                    {t("empire:sections.ethics")}
                  </div>
                  <div className="flex flex-wrap gap-1.5">
                    {snapshot.ethics.length > 0 ? (
                      snapshot.ethics.map((ethic) => (
                        <Badge
                          key={ethic}
                          variant="info"
                          className="h-6 gap-1 font-mono text-meta text-foreground"
                        >
                          <span className="size-1.5 rounded-full bg-info" />
                          <span>{ethic}</span>
                        </Badge>
                      ))
                    ) : (
                      <span className="text-body text-muted-foreground">
                        {t("empire:empty.ethics")}
                      </span>
                    )}
                  </div>
                </div>

                <div className="space-y-1">
                  <div className="text-meta font-medium text-muted-foreground">
                    {t("empire:sections.civics")}
                  </div>
                  <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                    {snapshot.government.civics.length > 0 ? (
                      snapshot.government.civics.map((civic) => (
                        <div
                          key={civic}
                          className="flex items-center justify-between rounded bg-surface px-2.5 py-1.5 text-body"
                        >
                          <span className="truncate font-mono text-foreground">
                            {civic}
                          </span>
                          <Badge
                            variant="secondary"
                            className="h-4 shrink-0 px-1 text-meta"
                          >
                            {t("empire:civicBadge")}
                          </Badge>
                        </div>
                      ))
                    ) : (
                      <span className="text-body text-muted-foreground">
                        {t("empire:empty.civics")}
                      </span>
                    )}
                  </div>
                </div>
              </TabsContent>

              <TabsContent value="traits" className="space-y-2 pt-2">
                <div className="text-meta font-medium text-muted-foreground">
                  {t("empire:sections.traits")}
                </div>
                <div className="flex flex-wrap gap-1.5">
                  {snapshot.founderSpecies.traits.length > 0 ? (
                    snapshot.founderSpecies.traits.map((trait) => (
                      <IdBadge key={trait} value={trait} />
                    ))
                  ) : (
                    <span className="text-body text-muted-foreground">
                      {t("empire:empty.traits")}
                    </span>
                  )}
                </div>
              </TabsContent>

              <TabsContent value="ascension" className="space-y-2.5 pt-2">
                <div className="space-y-1">
                  <div className="text-meta font-medium text-muted-foreground">
                    {t("empire:sections.ascensionPerks")}
                  </div>
                  <ExpandableIdList
                    items={snapshot.ascensionPerks}
                    emptyText={t("empire:empty.ascensionPerks")}
                    showMoreLabel={(count) =>
                      t("empire:showMore", { count })
                    }
                    showLessLabel={t("empire:showLess")}
                  />
                </div>

                <div className="space-y-1">
                  <div className="text-meta font-medium text-muted-foreground">
                    {t("empire:sections.traditions")}
                  </div>
                  <ExpandableIdList
                    items={snapshot.traditions}
                    emptyText={t("empire:empty.traditions")}
                    showMoreLabel={(count) =>
                      t("empire:showMore", { count })
                    }
                    showLessLabel={t("empire:showLess")}
                  />
                </div>
              </TabsContent>
            </Tabs>
          </div>
        ) : (
          <div className="py-16 text-center text-body text-muted-foreground">
            {t("empire:selectSaveHint")}
          </div>
        )}
      </Panel>
    </div>
  );
}

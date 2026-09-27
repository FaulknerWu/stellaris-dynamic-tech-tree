import { Channel, invoke } from "@tauri-apps/api/core";

import type {
  BootstrapDataDto,
  EnvironmentDto,
  GenerationProgressDto,
  GenerationRequestDto,
  GenerationResultDto,
  InspectSaveRequestDto,
  InspectedSaveDto,
  ResolveEnvironmentRequestDto,
  SaveLibraryDto,
  ScanSaveLibraryRequestDto,
} from "./bindings";
import { normalizeError } from "./errors";

export async function getBootstrapData(): Promise<BootstrapDataDto> {
  try {
    return await invoke<BootstrapDataDto>("get_bootstrap_data");
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function resolveEnvironment(
  request: ResolveEnvironmentRequestDto,
): Promise<EnvironmentDto> {
  try {
    return await invoke<EnvironmentDto>("resolve_environment", { request });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function scanSaveLibrary(
  request: ScanSaveLibraryRequestDto,
): Promise<SaveLibraryDto> {
  try {
    return await invoke<SaveLibraryDto>("scan_save_library", { request });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function inspectSave(
  request: InspectSaveRequestDto,
): Promise<InspectedSaveDto> {
  try {
    return await invoke<InspectedSaveDto>("inspect_save", { request });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function runGeneration(
  request: GenerationRequestDto,
  onProgress?: (progress: GenerationProgressDto) => void,
): Promise<GenerationResultDto> {
  try {
    const channel = new Channel<GenerationProgressDto>();
    if (onProgress) {
      channel.onmessage = onProgress;
    }
    return await invoke<GenerationResultDto>("run_generation", {
      request,
      onProgress: channel,
    });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function cancelGeneration(): Promise<boolean> {
  try {
    return await invoke<boolean>("cancel_generation");
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function openOutputDirectory(): Promise<void> {
  try {
    await invoke("open_output_directory");
  } catch (error) {
    throw normalizeError(error);
  }
}

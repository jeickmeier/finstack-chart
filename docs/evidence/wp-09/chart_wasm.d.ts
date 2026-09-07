/* tslint:disable */
/* eslint-disable */

/**
 * Owned chart session; every entry point preserves exact decimal-string wire identities.
 */
export class Chart {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Apply a revision-fenced action and return its outcome JSON.
     */
    action(input: string): string;
    /**
     * Canonical chart definition round-trip.
     */
    definition(): string;
    /**
     * Idempotent payload disposal. Further operations return CHART_DISPOSED_HANDLE errors.
     */
    dispose(): void;
    /**
     * Copy the supplied JSON/font payloads and construct one headless shared-core chart.
     */
    constructor(definition: string, data: string, profile: string, font: Uint8Array);
    /**
     * Restore state using a canonical decimal current-state revision fence.
     */
    restore_state(input: string, expected: string): void;
    /**
     * Return owned versioned scene JSON with numeric primitives and stable targets.
     */
    scene(): string;
    /**
     * Return owned semantic JSON.
     */
    semantics(): string;
    /**
     * Exact state snapshot envelope.
     */
    state(): string;
    /**
     * Basic SVG proof using the shared publication encoder, returned as an owned Uint8Array.
     */
    svg(): Uint8Array;
    /**
     * Apply a versioned transaction and return its typed outcome JSON.
     */
    transaction(input: string): string;
}

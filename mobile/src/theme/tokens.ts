import rawTokens from '../../../design/tokens.json';
import type { SemanticColors, TypographyToken } from './types';

interface DesignTokenContract {
  version: string;
  name: string;
  color: {
    light: SemanticColors;
    dark: SemanticColors;
  };
  space: Record<'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' | 'xxl' | 'section' | 'hero', number>;
  radius: Record<'sm' | 'md' | 'lg' | 'xl' | 'pill', number>;
  size: Record<'touch' | 'touchSafety' | 'input' | 'icon' | 'contentMax' | 'formMax', number>;
  motion: Record<'instant' | 'fast' | 'standard' | 'slow', number>;
  typography: Record<'display' | 'h1' | 'h2' | 'h3' | 'body' | 'bodyStrong' | 'label' | 'caption' | 'metric', TypographyToken>;
}

/**
 * Typed, immutable adapter over the canonical cross-platform token file.
 * No visual value is redefined in the mobile application.
 */
export const designTokens = rawTokens as unknown as DesignTokenContract;

export const getSemanticColors = (scheme: 'light' | 'dark'): SemanticColors =>
  designTokens.color[scheme];

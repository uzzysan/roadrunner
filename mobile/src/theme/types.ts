import type { Theme as NavigationTheme } from '@react-navigation/native';
import type { MD3Theme } from 'react-native-paper';

export type ThemePreference = 'system' | 'light' | 'dark';
export type ResolvedColorScheme = 'light' | 'dark';

export interface SemanticColors {
  background: string;
  surface: string;
  surfaceRaised: string;
  surfaceMuted: string;
  text: string;
  textSecondary: string;
  textMuted: string;
  border: string;
  borderStrong: string;
  primary: string;
  primaryHover: string;
  primaryPressed: string;
  onPrimary: string;
  primaryContainer: string;
  onPrimaryContainer: string;
  accent: string;
  onAccent: string;
  focus: string;
  success: string;
  successContainer: string;
  onSuccessContainer: string;
  warning: string;
  warningContainer: string;
  onWarningContainer: string;
  danger: string;
  onDanger: string;
  dangerContainer: string;
  onDangerContainer: string;
  info: string;
  infoContainer: string;
  onInfoContainer: string;
  disabled: string;
  onDisabled: string;
  inverse: string;
  onInverse: string;
}

export interface AppColors extends SemanticColors {
  /** Compatibility aliases. New code should use surface, danger and textMuted. */
  card: string;
  error: string;
  textTertiary: string;
  overlay: string;
}

export interface TypographyToken {
  size: number;
  lineHeight: number;
  weight: 400 | 600 | 700;
}

export interface AppTheme {
  name: string;
  version: string;
  scheme: ResolvedColorScheme;
  dark: boolean;
  colors: AppColors;
  space: Readonly<Record<'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' | 'xxl' | 'section' | 'hero', number>>;
  radius: Readonly<Record<'sm' | 'md' | 'lg' | 'xl' | 'pill', number>>;
  size: Readonly<Record<'touch' | 'touchSafety' | 'input' | 'icon' | 'contentMax' | 'formMax', number>>;
  motion: Readonly<Record<'instant' | 'fast' | 'standard' | 'slow', number>>;
  typography: Readonly<Record<'display' | 'h1' | 'h2' | 'h3' | 'body' | 'bodyStrong' | 'label' | 'caption' | 'metric', TypographyToken>>;
  paperTheme: MD3Theme;
  navigationTheme: NavigationTheme;
}

export interface SerializableThemeTokens {
  name: string;
  version: string;
  scheme: ResolvedColorScheme;
  colors: SemanticColors;
  space: AppTheme['space'];
  radius: AppTheme['radius'];
  size: AppTheme['size'];
  motion: AppTheme['motion'];
  typography: AppTheme['typography'];
  reducedMotion: boolean;
}

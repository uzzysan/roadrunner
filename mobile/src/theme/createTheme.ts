import {
  DarkTheme as NavigationDarkTheme,
  DefaultTheme as NavigationDefaultTheme,
} from '@react-navigation/native';
import { MD3DarkTheme, MD3LightTheme } from 'react-native-paper';
import { designTokens, getSemanticColors } from './tokens';
import type {
  AppColors,
  AppTheme,
  ResolvedColorScheme,
  SerializableThemeTokens,
} from './types';

const withCompatibilityAliases = (scheme: ResolvedColorScheme): AppColors => {
  const colors = getSemanticColors(scheme);
  return {
    ...colors,
    card: colors.surface,
    error: colors.danger,
    textTertiary: colors.textMuted,
    overlay: 'rgba(0, 0, 0, 0.48)',
  };
};

export const createAppTheme = (scheme: ResolvedColorScheme): AppTheme => {
  const colors = withCompatibilityAliases(scheme);
  const basePaperTheme = scheme === 'dark' ? MD3DarkTheme : MD3LightTheme;
  const baseNavigationTheme = scheme === 'dark' ? NavigationDarkTheme : NavigationDefaultTheme;

  const paperTheme = {
    ...basePaperTheme,
    dark: scheme === 'dark',
    roundness: designTokens.radius.md,
    colors: {
      ...basePaperTheme.colors,
      primary: colors.primary,
      onPrimary: colors.onPrimary,
      primaryContainer: colors.primaryContainer,
      onPrimaryContainer: colors.onPrimaryContainer,
      secondary: colors.textSecondary,
      onSecondary: colors.surface,
      secondaryContainer: colors.surfaceMuted,
      onSecondaryContainer: colors.text,
      background: colors.background,
      onBackground: colors.text,
      surface: colors.surface,
      onSurface: colors.text,
      surfaceVariant: colors.surfaceMuted,
      onSurfaceVariant: colors.textSecondary,
      outline: colors.borderStrong,
      outlineVariant: colors.border,
      error: colors.danger,
      onError: colors.onDanger,
      errorContainer: colors.dangerContainer,
      onErrorContainer: colors.onDangerContainer,
      surfaceDisabled: colors.disabled,
      onSurfaceDisabled: colors.onDisabled,
      inverseSurface: colors.inverse,
      inverseOnSurface: colors.onInverse,
      inversePrimary: colors.primary,
      elevation: {
        ...basePaperTheme.colors.elevation,
        level0: 'transparent',
        level1: colors.surface,
        level2: colors.surfaceRaised,
        level3: colors.surfaceRaised,
        level4: colors.surfaceRaised,
        level5: colors.surfaceRaised,
      },
    },
  };

  const navigationTheme = {
    ...baseNavigationTheme,
    dark: scheme === 'dark',
    colors: {
      ...baseNavigationTheme.colors,
      primary: colors.primary,
      background: colors.background,
      card: colors.surface,
      text: colors.text,
      border: colors.border,
      notification: colors.danger,
    },
  };

  return {
    name: designTokens.name,
    version: designTokens.version,
    scheme,
    dark: scheme === 'dark',
    colors,
    space: designTokens.space,
    radius: designTokens.radius,
    size: designTokens.size,
    motion: designTokens.motion,
    typography: designTokens.typography,
    paperTheme,
    navigationTheme,
  };
};

export const lightAppTheme = createAppTheme('light');
export const darkAppTheme = createAppTheme('dark');

export const serializeThemeTokens = (
  theme: AppTheme,
  reducedMotion: boolean,
): SerializableThemeTokens => ({
  name: theme.name,
  version: theme.version,
  scheme: theme.scheme,
  colors: getSemanticColors(theme.scheme),
  space: theme.space,
  radius: theme.radius,
  size: theme.size,
  motion: reducedMotion
    ? { instant: 0, fast: 0, standard: 0, slow: 0 }
    : theme.motion,
  typography: theme.typography,
  reducedMotion,
});

import { useTheme } from './useTheme';

export const useReducedMotion = (): boolean => useTheme().reducedMotion;

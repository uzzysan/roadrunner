import { useContext } from 'react';
import { ThemeContext } from '../theme/ThemeProvider';

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used inside RoadRunnerThemeProvider');
  }
  return context;
}

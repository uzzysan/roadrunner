import React from 'react';
import { StatusBar } from 'expo-status-bar';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { PaperProvider } from 'react-native-paper';

// i18n
import './src/i18n';

// Navigation
import { AppNavigator } from './src/navigation/AppNavigator';

export default function App() {
  return (
    <SafeAreaProvider>
      <PaperProvider>
        <StatusBar style="auto" />
        <AppNavigator />
      </PaperProvider>
    </SafeAreaProvider>
  );
}

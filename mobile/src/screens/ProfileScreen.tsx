import React from 'react';
import { View, StyleSheet } from 'react-native';
import { Text, useTheme, Button } from 'react-native-paper';

export default function ProfileScreen() {
  const theme = useTheme();
  return (
    <View style={[styles.container, { backgroundColor: theme.colors.background }]}>
      <Text variant="headlineMedium" style={{ color: theme.colors.onBackground }}>
        👤 Profil
      </Text>
      <Text style={{ color: theme.colors.onSurfaceVariant, marginTop: 16, marginBottom: 24 }}>
        Tu będą ustawienia konta
      </Text>
      <Button mode="contained" buttonColor={theme.colors.primary}>
        Zaloguj się
      </Button>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24,
  },
});

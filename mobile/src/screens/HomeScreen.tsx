import React from 'react';
import { View, StyleSheet, Image, ScrollView } from 'react-native';
import { Text, Card, Button, useTheme } from 'react-native-paper';

export default function HomeScreen() {
  const theme = useTheme();

  return (
    <ScrollView style={[styles.container, { backgroundColor: theme.colors.background }]}>
      <View style={styles.header}>
        <Image
          source={require('../assets/logo-transparent.png')}
          style={styles.logo}
          resizeMode="contain"
        />
        <Text variant="headlineMedium" style={{ color: theme.colors.onBackground }}>
          Witaj w RoadRunner!
        </Text>
        <Text variant="bodyLarge" style={{ color: theme.colors.onSurfaceVariant, marginTop: 8 }}>
          Twój system transportu zbiorowego
        </Text>
      </View>

      <View style={styles.cards}>
        <Card style={[styles.card, { backgroundColor: theme.colors.surface }]}>
          <Card.Content>
            <Text variant="titleLarge" style={{ color: theme.colors.onSurface }}>
              🚌 Mój autobus
            </Text>
            <Text variant="bodyMedium" style={{ color: theme.colors.onSurfaceVariant, marginTop: 8 }}>
              Sprawdź gdzie jest Twój autobus w czasie rzeczywistym
            </Text>
          </Card.Content>
          <Card.Actions>
            <Button mode="contained" buttonColor={theme.colors.primary}>
              Śledź
            </Button>
          </Card.Actions>
        </Card>

        <Card style={[styles.card, { backgroundColor: theme.colors.surface }]}>
          <Card.Content>
            <Text variant="titleLarge" style={{ color: theme.colors.onSurface }}>
              🎫 Moje bilety
            </Text>
            <Text variant="bodyMedium" style={{ color: theme.colors.onSurfaceVariant, marginTop: 8 }}>
              Zarządzaj swoimi biletami i kodami QR
            </Text>
          </Card.Content>
          <Card.Actions>
            <Button mode="contained" buttonColor={theme.colors.secondary}>
              Zobacz
            </Button>
          </Card.Actions>
        </Card>

        <Card style={[styles.card, { backgroundColor: theme.colors.surface }]}>
          <Card.Content>
            <Text variant="titleLarge" style={{ color: theme.colors.onSurface }}>
              👶 Dzieci
            </Text>
            <Text variant="bodyMedium" style={{ color: theme.colors.onSurfaceVariant, marginTop: 8 }}>
              Śledź bezpieczeństwo dzieci w transporcie szkolnym
            </Text>
          </Card.Content>
          <Card.Actions>
            <Button mode="contained" buttonColor={theme.colors.tertiary} >
              Sprawdź
            </Button>
          </Card.Actions>
        </Card>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  header: {
    alignItems: 'center',
    padding: 24,
    paddingTop: 16,
  },
  logo: {
    width: 200,
    height: 120,
    marginBottom: 16,
  },
  cards: {
    padding: 16,
    gap: 16,
  },
  card: {
    marginBottom: 12,
  },
});

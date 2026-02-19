import React from 'react';
import { View, StyleSheet, Image, ScrollView } from 'react-native';
import { Text, Card, Button, List, useTheme } from 'react-native-paper';

export default function HomeScreen() {
  const theme = useTheme();

  // Mock data - w przyszłości z API
  const nearestStop = {
    name: 'Plac Wolności',
    distance: '150 m',
    lines: [
      { number: '15', direction: 'Centrum', arrival: '3 min' },
      { number: '22', direction: 'Dworzec', arrival: '7 min' },
      { number: '5', direction: 'Szpital', arrival: '12 min' },
    ],
  };

  return (
    <ScrollView style={[styles.container, { backgroundColor: theme.colors.background }]}>
      <View style={styles.header}>
        <Image
          source={require('../assets/logo-transparent.png')}
          style={styles.logo}
          resizeMode="contain"
        />
      </View>

      {/* Najbliższy przystanek */}
      <Card style={[styles.card, { backgroundColor: theme.colors.surface }]}>
        <Card.Title
          title={}
          subtitle={}
          titleStyle={{ color: theme.colors.onSurface }}
          subtitleStyle={{ color: theme.colors.onSurfaceVariant }}
        />
        <Card.Content>
          <Text variant="titleMedium" style={{ color: theme.colors.onSurface, marginBottom: 8 }}>
            Nadjeżdżające autobusy:
          </Text>
          {nearestStop.lines.map((line, index) => (
            <View key={index} style={styles.lineRow}>
              <View style={[styles.lineNumber, { backgroundColor: theme.colors.primary }]}>
                <Text style={styles.lineNumberText}>{line.number}</Text>
              </View>
              <View style={styles.lineInfo}>
                <Text style={{ color: theme.colors.onSurface }}>{line.direction}</Text>
                <Text style={{ color: theme.colors.primary, fontWeight: 'bold' }}>{line.arrival}</Text>
              </View>
            </View>
          ))}
        </Card.Content>
        <Card.Actions>
          <Button 
            mode="contained" 
            buttonColor={theme.colors.primary}
            onPress={() => {}}
          >
            Znajdź przystanek
          </Button>
          <Button 
            mode="outlined" 
            textColor={theme.colors.primary}
            onPress={() => {}}
          >
            Pełny rozkład
          </Button>
        </Card.Actions>
      </Card>

      {/* Dla zalogowanych - Bilety */}
      <Card style={[styles.card, { backgroundColor: theme.colors.surface }]}>
        <Card.Title
          title="🎫 Bilety" 
          subtitle="Zaloguj się, aby zobaczyć bilety" 
          titleStyle={{ color: theme.colors.onSurface }}
          subtitleStyle={{ color: theme.colors.onSurfaceVariant }}
        />
        <Card.Actions>
          <Button mode="contained" buttonColor={theme.colors.secondary}>
            Zaloguj się
          </Button>
        </Card.Actions>
      </Card>

      {/* Dla rodziców - Dzieci */}
      <Card style={[styles.card, { backgroundColor: theme.colors.surface }]}>
        <Card.Title
          title="👶 Dzieci" 
          subtitle="Widoczne po zalogowaniu jako rodzic" 
          titleStyle={{ color: theme.colors.onSurface }}
          subtitleStyle={{ color: theme.colors.onSurfaceVariant }}
        />
        <Card.Content>
          <Text style={{ color: theme.colors.onSurfaceVariant }}>
            Śledź bezpieczeństwo dzieci w transporcie szkolnym. 
            Otrzymuj powiadomienia o wsiadaniu i wysiadaniu.
          </Text>
        </Card.Content>
      </Card>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  header: {
    alignItems: 'center',
    padding: 16,
  },
  logo: {
    width: 180,
    height: 100,
  },
  card: {
    margin: 12,
    marginTop: 0,
  },
  lineRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  lineNumber: {
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: 'center',
    alignItems: 'center',
  },
  lineNumberText: {
    color: 'white',
    fontWeight: 'bold',
    fontSize: 16,
  },
  lineInfo: {
    flex: 1,
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginLeft: 12,
  },
});

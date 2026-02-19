import React from 'react';
import { View, StyleSheet, Image, ScrollView, Dimensions } from 'react-native';
import { Text, Card, Button, useTheme } from 'react-native-paper';
import { LinearGradient } from 'expo-linear-gradient';
import { gradients } from '../theme/colors';

const { height } = Dimensions.get('window');

export default function HomeScreen() {
  const theme = useTheme();
  const isDark = theme.dark;

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
    <View style={styles.container}>
      {/* Główna zawartość */}
      <ScrollView 
        style={styles.scrollView}
        contentContainerStyle={styles.content}
        showsVerticalScrollIndicator={false}
      >
        {/* Logo */}
        <View style={styles.logoContainer}>
          <Image
            source={require('../assets/logo-transparent.png')}
            style={styles.logo}
            resizeMode="contain"
          />
        </View>

        {/* Karta przystanku */}
        <Card 
          style={[
            styles.card, 
            { 
              backgroundColor: theme.colors.surface,
              shadowColor: isDark ? '#000' : '#E53935',
              shadowOpacity: 0.1,
            }
          ]}
        >
          <Card.Title
            title={nearestStop.name}
            subtitle={nearestStop.distance + ' • Najbliższy przystanek'}
            titleStyle={{ 
              color: theme.colors.onSurface, 
              fontSize: 20, 
              fontWeight: '600' 
            }}
            subtitleStyle={{ color: theme.colors.onSurfaceVariant }}
          />
          
          <Card.Content style={styles.linesContainer}>
            {nearestStop.lines.map((line, index) => (
              <View key={index} style={styles.lineRow}>
                <View style={[
                  styles.lineBadge, 
                  { backgroundColor: theme.colors.primary }
                ]}>
                  <Text style={styles.lineNumber}>{line.number}</Text>
                </View>
                <View style={styles.lineDetails}>
                  <Text style={{ color: theme.colors.onSurface, fontSize: 16 }}>
                    {line.direction}
                  </Text>
                  <Text style={{ 
                    color: theme.colors.primary, 
                    fontWeight: '700',
                    fontSize: 14 
                  }}>
                    {line.arrival}
                  </Text>
                </View>
              </View>
            ))}
          </Card.Content>

          <Card.Actions style={styles.cardActions}>
            <Button 
              mode="contained" 
              buttonColor={theme.colors.primary}
              style={styles.button}
            >
              Pełny rozkład
            </Button>
          </Card.Actions>
        </Card>

        {/* Karta biletów */}
        <Card style={[
          styles.card, 
          { 
            backgroundColor: theme.colors.surface,
            shadowColor: isDark ? '#000' : '#1A1A2E',
            shadowOpacity: 0.08,
          }
        ]}>
          <Card.Title
            title="Moje bilety" 
            titleStyle={{ color: theme.colors.onSurface, fontSize: 18 }}
          />
          <Card.Content>
            <Text style={{ color: theme.colors.onSurfaceVariant }}>
              Zaloguj się, aby zobaczyć swoje bilety QR
            </Text>
          </Card.Content>
          <Card.Actions>
            <Button 
              mode="outlined" 
              textColor={theme.colors.primary}
              style={styles.button}
            >
              Zaloguj się
            </Button>
          </Card.Actions>
        </Card>

        {/* Miejsce na kolejne karty */}
        <View style={{ height: 100 }} />
      </ScrollView>

      {/* Gradient w dolnej części */}
      <LinearGradient
        colors={isDark ? gradients.dark : gradients.light}
        style={styles.gradient}
        locations={[0, 0.5, 1]}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  scrollView: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingTop: 60, // Miejsce na przezroczysty header
  },
  logoContainer: {
    alignItems: 'center',
    marginBottom: 20,
  },
  logo: {
    width: 160,
    height: 90,
  },
  card: {
    marginBottom: 16,
    borderRadius: 16,
    elevation: 4,
    shadowOffset: { width: 0, height: 2 },
    shadowRadius: 8,
  },
  linesContainer: {
    paddingTop: 8,
  },
  lineRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E020',
  },
  lineBadge: {
    width: 44,
    height: 44,
    borderRadius: 12,
    justifyContent: 'center',
    alignItems: 'center',
  },
  lineNumber: {
    color: 'white',
    fontWeight: 'bold',
    fontSize: 18,
  },
  lineDetails: {
    flex: 1,
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginLeft: 16,
  },
  cardActions: {
    padding: 16,
    paddingTop: 8,
  },
  button: {
    borderRadius: 8,
  },
  gradient: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    height: height * 0.33,
    zIndex: -1,
  },
});

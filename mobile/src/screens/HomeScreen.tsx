import React from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity } from 'react-native';
import { useTranslation } from 'react-i18next';
import { useAuthStore } from '../store/authStore';

export function HomeScreen() {
  const { t } = useTranslation();
  const { user } = useAuthStore();

  return (
    <ScrollView style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.greeting}>
          Witaj, {user?.firstName || 'Użytkowniku'}!
        </Text>
        <Text style={styles.subtitle}>
          RoadRunner - System Transportu
        </Text>
      </View>

      <View style={styles.cardsContainer}>
        <TouchableOpacity style={styles.card}>
          <Text style={styles.cardTitle}>Kup bilet</Text>
          <Text style={styles.cardDescription}>
            Szybki zakup biletu na przejazd
          </Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.card}>
          <Text style={styles.cardTitle}>Śledź pojazd</Text>
          <Text style={styles.cardDescription}>
            Sprawdź lokalizację autobusu w czasie rzeczywistym
          </Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.card}>
          <Text style={styles.cardTitle}>Moje bilety</Text>
          <Text style={styles.cardDescription}>
            Zarządzaj swoimi biletami
          </Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8FAFC',
  },
  header: {
    padding: 20,
    backgroundColor: '#2563EB',
  },
  greeting: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
  },
  subtitle: {
    fontSize: 16,
    color: '#DBEAFE',
    marginTop: 4,
  },
  cardsContainer: {
    padding: 16,
  },
  card: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 20,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#0F172A',
    marginBottom: 8,
  },
  cardDescription: {
    fontSize: 14,
    color: '#64748B',
  },
});

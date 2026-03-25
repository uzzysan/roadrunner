import React from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity, Alert } from 'react-native';
import { useTranslation } from 'react-i18next';
import { useAuthStore } from '../store/authStore';
import { authApi } from '../api/client';

export function ProfileScreen() {
  const { t } = useTranslation();
  const { user, logout, refreshToken } = useAuthStore();

  const handleLogout = async () => {
    try {
      if (refreshToken) {
        await authApi.logout(refreshToken);
      }
      logout();
    } catch (error) {
      console.error('Logout error:', error);
      logout(); // Logout locally even if API fails
    }
  };

  const confirmLogout = () => {
    Alert.alert(
      'Wylogowanie',
      'Czy na pewno chcesz się wylogować?',
      [
        { text: 'Anuluj', style: 'cancel' },
        { text: 'Wyloguj', onPress: handleLogout, style: 'destructive' },
      ]
    );
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.header}>
        <View style={styles.avatar}>
          <Text style={styles.avatarText}>
            {user?.firstName?.[0]}{user?.lastName?.[0]}
          </Text>
        </View>
        <Text style={styles.name}>
          {user?.firstName} {user?.lastName}
        </Text>
        <Text style={styles.email}>{user?.email}</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Konto</Text>

        <TouchableOpacity style={styles.menuItem}>
          <Text style={styles.menuItemText}>Edytuj profil</Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.menuItem}>
          <Text style={styles.menuItemText}>Zmień hasło</Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.menuItem}>
          <Text style={styles.menuItemText}>Ustawienia MFA</Text>
        </TouchableOpacity>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Aplikacja</Text>

        <TouchableOpacity style={styles.menuItem}>
          <Text style={styles.menuItemText}>Język</Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.menuItem}>
          <Text style={styles.menuItemText}>Powiadomienia</Text>
        </TouchableOpacity>
      </View>

      <TouchableOpacity style={styles.logoutButton} onPress={confirmLogout}>
        <Text style={styles.logoutButtonText}>Wyloguj się</Text>
      </TouchableOpacity>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8FAFC',
  },
  header: {
    backgroundColor: '#2563EB',
    padding: 32,
    alignItems: 'center',
  },
  avatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#fff',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 16,
  },
  avatarText: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#2563EB',
  },
  name: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 4,
  },
  email: {
    fontSize: 16,
    color: '#DBEAFE',
  },
  section: {
    backgroundColor: '#fff',
    marginTop: 16,
    paddingVertical: 8,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#64748B',
    paddingHorizontal: 16,
    paddingVertical: 8,
    textTransform: 'uppercase',
  },
  menuItem: {
    paddingHorizontal: 16,
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#E2E8F0',
  },
  menuItemText: {
    fontSize: 16,
    color: '#0F172A',
  },
  logoutButton: {
    backgroundColor: '#EF4444',
    margin: 16,
    padding: 16,
    borderRadius: 8,
    alignItems: 'center',
  },
  logoutButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});

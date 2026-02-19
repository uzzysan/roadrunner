import React, { useState } from 'react';
import { StatusBar } from 'expo-status-bar';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { PaperProvider, IconButton } from 'react-native-paper';
import { NavigationContainer } from '@react-navigation/native';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { MaterialCommunityIcons } from '@expo/vector-icons';
import { useColorScheme } from 'react-native';

import { lightTheme, darkTheme } from './src/theme/colors';
import HomeScreen from './src/screens/HomeScreen';
import TicketsScreen from './src/screens/TicketsScreen';
import TrackingScreen from './src/screens/TrackingScreen';
import ProfileScreen from './src/screens/ProfileScreen';

const Tab = createBottomTabNavigator();

export default function App() {
  const systemColorScheme = useColorScheme();
  const [isDarkMode, setIsDarkMode] = useState(systemColorScheme === 'dark');
  
  const theme = isDarkMode ? darkTheme : lightTheme;

  const toggleTheme = () => {
    setIsDarkMode(!isDarkMode);
  };

  return (
    <SafeAreaProvider>
      <PaperProvider theme={theme}>
        <NavigationContainer>
          <Tab.Navigator
            screenOptions={({ route }) => ({
              tabBarIcon: ({ focused, color, size }) => {
                let iconName: string;

                if (route.name === 'Home') {
                  iconName = focused ? 'home' : 'home-outline';
                } else if (route.name === 'Tickets') {
                  iconName = focused ? 'ticket' : 'ticket-outline';
                } else if (route.name === 'Tracking') {
                  iconName = focused ? 'map-marker' : 'map-marker-outline';
                } else if (route.name === 'Profile') {
                  iconName = focused ? 'account' : 'account-outline';
                } else {
                  iconName = 'help';
                }

                return <MaterialCommunityIcons name={iconName as any} size={size} color={color} />;
              },
              tabBarActiveTintColor: theme.colors.primary,
              tabBarInactiveTintColor: theme.colors.onSurfaceVariant,
              tabBarStyle: {
                backgroundColor: theme.colors.surface,
                borderTopWidth: 0,
                elevation: 0,
                shadowOpacity: 0,
              },
              // Przezroczysty header
              headerTransparent: true,
              headerStyle: {
                backgroundColor: 'transparent',
                elevation: 0,
                shadowOpacity: 0,
              },
              headerTintColor: theme.colors.onHeader,
              headerRight: () => (
                <IconButton
                  icon={isDarkMode ? 'white-balance-sunny' : 'moon-waning-crescent'}
                  iconColor={theme.colors.onHeader}
                  onPress={toggleTheme}
                  style={{ marginRight: 8 }}
                />
              ),
            })}
          >
            <Tab.Screen 
              name="Home" 
              component={HomeScreen}
              options={{ title: '' }}  // Pusty tytuł
            />
            <Tab.Screen 
              name="Tickets" 
              component={TicketsScreen}
              options={{ title: '' }}
            />
            <Tab.Screen 
              name="Tracking" 
              component={TrackingScreen}
              options={{ title: '' }}
            />
            <Tab.Screen 
              name="Profile" 
              component={ProfileScreen}
              options={{ title: '' }}
            />
          </Tab.Navigator>
        </NavigationContainer>
        <StatusBar style={isDarkMode ? 'light' : 'dark'} />
      </PaperProvider>
    </SafeAreaProvider>
  );
}

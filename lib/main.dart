import 'package:daily_wallpaper_images/bing.dart';
import 'package:daily_wallpaper_images/nasa.dart';
import 'package:daily_wallpaper_images/notifications.dart';
import 'package:daily_wallpaper_images/spotlight.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:flutter/material.dart';
import 'package:rinf/rinf.dart';

void main() async {
  await initializeRust(assignRustSignal);
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Daily Wallpaper Images',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blueGrey,
        ),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blueGrey,
          brightness: Brightness.dark,
        ),
      ),
      home: const MyHomePage(),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  int _selectedSource = 0;
  final services = [
    "Bing Image of the Day",
    "NASA Image of the Day",
    "Windows Spotlight"
  ];

  void _onItemTapped(int index) {
    setState(() {
      _selectedSource = index;
    });
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    var drawerDestinations = <Widget>[];
    for (final (i, item) in services.indexed) {
      drawerDestinations.add(ListTile(
        title: Text(item),
        onTap: () {
          _onItemTapped(i);
          Navigator.pop(context);
        },
      ));
    }

    return Scaffold(
      appBar: AppBar(
        backgroundColor: colorScheme.primaryContainer,
        foregroundColor: colorScheme.onPrimaryContainer,
        title: Text(services[_selectedSource]),
        leading: Builder(
          builder: (context) {
            return IconButton(
              icon: const Icon(Icons.menu),
              onPressed: () {
                Scaffold.of(context).openDrawer();
              },
            );
          },
        ),
        actions: [
          Builder(builder: (context) {
            return IconButton(
              onPressed: () {
                Scaffold.of(context).openEndDrawer();
              },
              icon: Icon(Icons.notifications),
            );
          }),
        ],
      ),
      drawer: Drawer(
        child: ListView(
          padding: EdgeInsets.zero,
          children: <Widget>[
                DrawerHeader(
                  decoration: BoxDecoration(
                    color: colorScheme.primaryContainer,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    spacing: 10.0,
                    children: [
                      Expanded(
                        child: Image.asset(
                          "assets/app_icon_no_bg.png",
                          color: colorScheme.onPrimaryContainer,
                        ),
                      ),
                      Text(
                        'Daily Image Sources',
                        style: TextStyle(
                          color: colorScheme.onPrimaryContainer,
                        ),
                      ),
                    ],
                  ),
                ),
              ] +
              drawerDestinations,
        ),
      ),
      body: [BingPage(), NasaPage(), SpotlightPage()][_selectedSource],
      floatingActionButton: _selectedSource == 2
          ? FloatingActionButton.small(
              onPressed: () {
                SpotlightReset().sendSignalToRust();
              },
              shape: CircleBorder(),
              tooltip: "I'm feeling lucky",
              child: Padding(
                padding: const EdgeInsets.all(3.0),
                child: Container(
                    decoration: BoxDecoration(
                      color: Colors.black,
                      shape: BoxShape.circle,
                    ),
                    child: SizedBox(
                      child: Align(
                        child: Text(
                          "8",
                          style: TextStyle(color: Colors.white),
                          textAlign: TextAlign.center,
                        ),
                      ),
                    )),
              ),
            )
          : null,
      endDrawer: NotificationCenter(),
    );
  }
}

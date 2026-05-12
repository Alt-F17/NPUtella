from pynput import keyboard


def on_press(key):
    try:
        print(f"Key Pressed: {key.char} | VK: {key.vk} | Scan: {key._scan}")
    except AttributeError:
        print(f"Special Key: {key} | Code: {getattr(key, 'value', 'N/A')}")


def on_release(key):
    if key == keyboard.Key.esc:
        print("\nExiting diagnostic...")
        return False
    return None


def main():
    print("--- Key Detection Diagnostic ---")
    print("Press keys to see data. Press 'ESC' to stop.\n")

    with keyboard.Listener(on_press=on_press, on_release=on_release) as listener:
        listener.join()


if __name__ == "__main__":
    main()

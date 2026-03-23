import logging
import typer
from tempit.core import TempitManager

app = typer.Typer(add_completion=False, help="CLI for the tempit application.")

def get_manager() -> TempitManager:
    """Helper pour initialiser le manager et gérer les erreurs globales."""
    try:
        return TempitManager()
    except (IOError, OSError) as e:
        logging.error("An error occurred: %s", e)
        raise typer.Exit(code=1)

@app.command("create")
def create_dir(name: str = typer.Argument(..., help="Name of the temporary directory.")):
    """Create a new temporary directory."""
    typer.echo(get_manager().create(name))

@app.command("init")
def init_shell(shell: str = typer.Argument(..., help="Shell name (e.g., bash, zsh).")):
    """Initialize Tempit in the current shell."""
    get_manager().init_shell(shell)

@app.command("list")
def list_dirs():
    """List all tracked temporary directories."""
    get_manager().print_directories()

@app.command("remove")
def remove_dir(number: int = typer.Argument(..., help="Number of the directory to remove.")):
    """Remove a tracked temporary directory by its number."""
    if not get_manager().remove(number):
        raise typer.Exit(code=1)

@app.command("clean-all")
def clean_all():
    """Remove all tracked temporary directories."""
    get_manager().clean_all_directories()

@app.command("path", hidden=True)
def get_path(number: int):
    """Get directory path by number."""
    typer.echo(get_manager().get_path_by_number(number))

def main():
    logging.basicConfig(level=logging.WARNING, format="%(levelname)s: %(message)s")
    app()

if __name__ == "__main__":
    main()

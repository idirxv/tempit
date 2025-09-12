"""Data models for the tempit application."""

from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Dict


@dataclass
class DirectoryInfo:
    """Data model for temporary directory information."""

    path: Path
    created: datetime
    prefix: str = "tempit"

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        data = asdict(self)
        data["created"] = self.created.isoformat()
        return data

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "DirectoryInfo":
        """Create instance from dictionary (JSON deserialization)."""
        data = data.copy()
        data["created"] = datetime.fromisoformat(data["created"])
        data["path"] = Path(data["path"])
        return cls(**data)


@dataclass
class DirectoryStats:
    """Runtime statistics for a directory (not stored in JSON)."""

    size_bytes: int
    human_size: str
    file_count: int
    dir_count: int
    age: str

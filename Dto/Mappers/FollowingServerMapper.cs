using DataBase.Entity;
using Riok.Mapperly.Abstractions;

namespace DtoMappers.Mappers;

[Mapper(EnumMappingStrategy = EnumMappingStrategy.ByName)]
public partial class FollowingServerMapper
{
    public static partial FollowingServerDto FollowingServerToDto(FollowingServer followingServer);

    public static partial FollowingServer FollowingServerToEntity(FollowingServerDto followingServerDto);
}
